//! Printing the sweep as a table and as mermaid.

use crate::chart;
use crate::sweep::Point;

/// The results table.
pub fn print_table(dataset: &str, points: &[Point]) {
    println!("\n---- {dataset} ----\n");
    println!(
        "{:<18} {:>9} {:>10} {:>9} {:>9} {:>9}",
        "configuration", "recall@10", "queries/s", "p50 ms", "p99 ms", "build s"
    );
    println!("{}", "-".repeat(70));
    for p in points {
        println!(
            "{:<18} {:>9.4} {:>10.0} {:>9.3} {:>9.3} {:>9.2}",
            p.label,
            p.recall,
            p.qps,
            p.p50.as_secs_f64() * 1000.0,
            p.p99.as_secs_f64() * 1000.0,
            p.build.as_secs_f64(),
        );
    }
}

/// The mermaid charts, ready to paste into a README or an issue.
pub fn print_charts(dataset: &str, points: &[Point]) {
    println!("\n---- charts ----\n");
    for family in ["hnsw", "ivf"] {
        print!(
            "{}",
            chart::fenced(&chart::recall_vs_qps(dataset, points, family))
        );
        println!();
    }
    println!();
    print!("{}", chart::fenced(&chart::tail_latency(dataset, points)));
    println!();
    print!("{}", chart::fenced(&chart::build_time(dataset, points)));
}

/// How to read the numbers, and what they do not say.
pub fn print_notes(points: &[Point]) {
    let exact: Vec<&Point> = points.iter().filter(|p| p.recall >= 0.9999).collect();

    println!("\n---- how to read this ----\n");
    println!("  Recall is measured against the dataset's own exhaustive ground");
    println!("  truth, so it is exact rather than relative to another index.");
    println!();
    if !exact.is_empty() {
        println!("  The exhaustive rows sit at recall 1.0 by construction — they");
        println!("  score every vector. They are the correctness reference and the");
        println!("  throughput floor, not a competitor to HNSW: the comparison that");
        println!("  matters is queries/s *at equal recall*.");
        println!();
    }
    println!("  QPS is single-threaded. A concurrent figure measures the runtime");
    println!("  and the core count as much as the index, and is not comparable");
    println!("  across reports that do not state both.");
    println!();
    println!("  p99 is reported because the mean hides what a serving system is");
    println!("  sized by: an index that is fast on average and occasionally");
    println!("  terrible is worse than a uniformly slower one.");
    println!();
    println!("  Not measured here: filtered search, concurrency, and recall under");
    println!("  deletes. Those are where vector databases differ most in practice,");
    println!("  and this harness says nothing about them yet.");
}
