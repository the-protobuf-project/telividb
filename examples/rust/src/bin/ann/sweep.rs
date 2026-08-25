//! Measuring one index configuration: recall, throughput, latency.

use crate::vecs::Dataset;
use std::time::{Duration, Instant};
use telividb_core::{Dim, Metric, Ordinal};
use telividb_index::VectorIndex;
use telividb_index::adapters::MemoryStore;

/// Neighbours retrieved and scored. The field compares at 10.
pub const K: usize = 10;

/// One configuration's measurement.
pub struct Point {
    /// Which index and at what setting, e.g. `hnsw ef=64`.
    pub label: String,
    /// Index family, so a chart can group its series.
    pub family: String,
    /// Mean recall@10 against exact ground truth.
    pub recall: f64,
    /// Queries per second, single-threaded.
    ///
    /// Single-threaded on purpose: it is the number that isolates the index.
    /// A concurrent figure measures the runtime and the machine's core count
    /// as much as the algorithm, and is not comparable across reports that do
    /// not state both.
    pub qps: f64,
    /// Median query latency.
    pub p50: Duration,
    /// 99th percentile query latency.
    ///
    /// Reported because the mean hides exactly what matters in a serving
    /// system: an index that is fast on average and occasionally terrible is
    /// worse than a uniformly slower one, and the mean cannot tell them apart.
    pub p99: Duration,
    /// Time to build the index.
    pub build: Duration,
}

/// Load a dataset's base vectors into a store.
///
/// `Dot` on pre-normalised vectors would be cosine; SIFT is raw and its ground
/// truth is Euclidean, so `L2` is what makes recall comparable to the
/// published numbers.
pub fn store_of(dataset: &Dataset) -> Result<MemoryStore, Box<dyn std::error::Error>> {
    let mut store = MemoryStore::new(Dim::new(dataset.dim as u32)?, Metric::L2);
    for vector in &dataset.base {
        store.push(vector)?;
    }
    Ok(store)
}

/// Time an index over every query and score it against the ground truth.
pub fn measure(
    label: &str,
    family: &str,
    dataset: &Dataset,
    store: &MemoryStore,
    index: &dyn VectorIndex,
    build: Duration,
) -> Result<Point, Box<dyn std::error::Error>> {
    let mut latencies = Vec::with_capacity(dataset.queries.len());
    let mut found = 0usize;
    let mut possible = 0usize;

    let started = Instant::now();
    for (i, query) in dataset.queries.iter().enumerate() {
        let at = Instant::now();
        let hits = index.search(store, query, K, None)?;
        latencies.push(at.elapsed());

        // Ground truth is ordered nearest-first, so the first K are the answer.
        let truth = match dataset.truth.get(i) {
            Some(t) => t,
            None => continue,
        };
        let wanted: Vec<u32> = truth.iter().take(K).copied().collect();
        possible += wanted.len();
        found += hits
            .iter()
            .filter(|hit| wanted.contains(&ordinal_row(hit.ordinal)))
            .count();
    }
    let elapsed = started.elapsed();

    latencies.sort_unstable();
    Ok(Point {
        label: label.to_owned(),
        family: family.to_owned(),
        recall: match possible {
            0 => 0.0,
            n => found as f64 / n as f64,
        },
        qps: dataset.queries.len() as f64 / elapsed.as_secs_f64().max(1e-9),
        p50: percentile(&latencies, 0.50),
        p99: percentile(&latencies, 0.99),
        build,
    })
}

/// A row number, as the ground truth expresses it.
fn ordinal_row(ordinal: Ordinal) -> u32 {
    ordinal.row()
}

/// The value at `fraction` through a sorted slice.
fn percentile(sorted: &[Duration], fraction: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    // Nearest-rank, clamped: with 100 queries the p99 is the 99th, and an
    // index one past the end would panic on a small run.
    let rank = ((sorted.len() as f64) * fraction).ceil() as usize;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}
