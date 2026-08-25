//! Rendering results as mermaid, so they can be published as-is.
//!
//! Mermaid rather than an image because the output is text: it diffs, it
//! survives a copy-paste into a README or an issue, and it needs no plotting
//! toolchain in the build. `xychart-beta` has no log axis and no true scatter,
//! so the classic recall-versus-QPS plot is expressed as a line whose x-axis
//! *labels* are the measured recalls — which works because recall is monotonic
//! in the search-breadth parameter being swept.

use crate::sweep::Point;

/// The recall-versus-throughput curve: the axis the field compares on.
///
/// Up and to the right is better — high recall at high throughput. A point
/// that is high on one and low on the other is the trade every ANN index
/// makes, and the curve is how you see the shape of it.
pub fn recall_vs_qps(dataset: &str, points: &[Point], family: &str) -> String {
    let frontier = pareto_frontier(points, family);
    if frontier.is_empty() {
        return String::new();
    }

    let labels: Vec<String> = frontier
        .iter()
        .map(|p| format!("\"{:.3}\"", p.recall))
        .collect();
    let values: Vec<String> = frontier.iter().map(|p| format!("{:.0}", p.qps)).collect();
    let ceiling = frontier.iter().map(|p| p.qps).fold(0.0, f64::max) * 1.1;

    format!(
        "xychart-beta\n    \
         title \"{dataset} — {family} recall@10 versus throughput\"\n    \
         x-axis \"recall@10\" [{}]\n    \
         y-axis \"queries per second\" 0 --> {ceiling:.0}\n    \
         line [{}]\n",
        labels.join(", "),
        values.join(", "),
    )
}

/// The best throughput achieved at each recall level.
///
/// What `ann-benchmarks` plots, and the only honest summary of a sweep: a
/// configuration that is beaten on *both* axes by another is never the right
/// choice, so showing it would only pad the curve. Sweeping `ef` past the
/// point where recall saturates produces exactly such points — slower, and no
/// more accurate.
///
/// Rounded to three decimals before grouping, because that is the precision
/// the chart shows; two configurations differing in the fourth decimal are the
/// same point to a reader.
fn pareto_frontier<'a>(points: &'a [Point], family: &str) -> Vec<&'a Point> {
    let mut best: Vec<&Point> = Vec::new();
    for point in points.iter().filter(|p| p.family == family) {
        let key = (point.recall * 1000.0).round() as i64;
        match best
            .iter_mut()
            .find(|p| ((p.recall * 1000.0).round() as i64) == key)
        {
            Some(existing) if existing.qps < point.qps => *existing = point,
            Some(_) => {}
            None => best.push(point),
        }
    }
    best.sort_by(|a, b| a.recall.total_cmp(&b.recall));
    best
}

/// Tail latency per configuration.
///
/// A bar chart rather than a line: these are separate configurations, not a
/// progression, and p99 is the number a serving system is actually sized by.
pub fn tail_latency(dataset: &str, points: &[Point]) -> String {
    if points.is_empty() {
        return String::new();
    }

    let labels: Vec<String> = points.iter().map(|p| format!("\"{}\"", p.label)).collect();
    let values: Vec<String> = points
        .iter()
        .map(|p| format!("{:.3}", p.p99.as_secs_f64() * 1000.0))
        .collect();
    let ceiling = points
        .iter()
        .map(|p| p.p99.as_secs_f64() * 1000.0)
        .fold(0.0, f64::max)
        * 1.15;

    format!(
        "xychart-beta\n    \
         title \"{dataset} — p99 query latency by configuration\"\n    \
         x-axis [{}]\n    \
         y-axis \"milliseconds (p99)\" 0 --> {ceiling:.3}\n    \
         bar [{}]\n",
        labels.join(", "),
        values.join(", "),
    )
}

/// Index build cost per configuration.
///
/// Worth publishing beside recall: an index that reaches 0.99 recall but takes
/// an hour to build is a different product from one that reaches 0.97 in a
/// minute, and the recall curve alone cannot show that.
pub fn build_time(dataset: &str, points: &[Point]) -> String {
    // One bar per distinct build; sweeping `ef_search` reuses a graph, so
    // repeating its build time once per point would misrepresent the cost.
    let mut seen: Vec<&Point> = Vec::new();
    for point in points {
        if !seen.iter().any(|p| p.family == point.family) {
            seen.push(point);
        }
    }
    if seen.is_empty() {
        return String::new();
    }

    let labels: Vec<String> = seen.iter().map(|p| format!("\"{}\"", p.family)).collect();
    let values: Vec<String> = seen
        .iter()
        .map(|p| format!("{:.2}", p.build.as_secs_f64()))
        .collect();
    let ceiling = seen
        .iter()
        .map(|p| p.build.as_secs_f64())
        .fold(0.0, f64::max)
        .max(0.01)
        * 1.15;

    format!(
        "xychart-beta\n    \
         title \"{dataset} — index build time\"\n    \
         x-axis [{}]\n    \
         y-axis \"seconds\" 0 --> {ceiling:.2}\n    \
         bar [{}]\n",
        labels.join(", "),
        values.join(", "),
    )
}

/// Wrap a chart in a fenced block, ready to paste into markdown.
pub fn fenced(chart: &str) -> String {
    match chart.is_empty() {
        true => String::new(),
        false => format!("```mermaid\n{chart}```\n"),
    }
}
