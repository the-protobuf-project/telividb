//! Command-line arguments for the recall harness.

use episteme_core::Metric;

/// How the synthetic corpus is shaped.
///
/// This is not a cosmetic knob. Uniformly random vectors in high dimensions
/// suffer concentration of measure: every pairwise distance converges on the
/// same value, the true nearest neighbour is barely nearer than the thousandth,
/// and *every* ANN method degrades — so recall measured on uniform data says
/// nothing about how an index performs on real embeddings.
///
/// Real embeddings, and the standard benchmark sets, have cluster structure and
/// a low intrinsic dimension. `Clustered` reproduces that and is the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distribution {
    /// Adversarial. Useful only to show how bad concentration gets.
    Uniform,
    /// Gaussian blobs around random centres — like real embeddings.
    Clustered,
}

pub struct Args {
    pub rows: usize,
    pub dim: usize,
    pub k: usize,
    pub queries: usize,
    pub ef: usize,
    pub metric: Metric,
    pub distribution: Distribution,
    pub clusters: usize,
    /// Nodes per parallel build batch. One disables batching.
    pub batch: usize,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            rows: 50_000,
            dim: 128,
            k: 10,
            queries: 100,
            ef: 64,
            metric: Metric::Cosine,
            distribution: Distribution::Clustered,
            clusters: 64,
            batch: 1,
        }
    }
}

pub fn parse() -> Args {
    let mut args = Args::default();
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i + 1 < raw.len() {
        let value = &raw[i + 1];
        match raw[i].as_str() {
            "--rows" => args.rows = value.parse().unwrap_or(args.rows),
            "--dim" => args.dim = value.parse().unwrap_or(args.dim),
            "--k" => args.k = value.parse().unwrap_or(args.k),
            "--queries" => args.queries = value.parse().unwrap_or(args.queries),
            "--ef" => args.ef = value.parse().unwrap_or(args.ef),
            "--distribution" => {
                args.distribution = match value.as_str() {
                    "uniform" => Distribution::Uniform,
                    _ => Distribution::Clustered,
                }
            }
            "--clusters" => args.clusters = value.parse().unwrap_or(args.clusters),
            "--batch" => args.batch = value.parse().unwrap_or(args.batch),
            "--metric" => {
                args.metric = match value.as_str() {
                    "dot" => Metric::Dot,
                    "l2" => Metric::L2,
                    _ => Metric::Cosine,
                }
            }
            other => eprintln!("ignoring unknown flag {other}"),
        }
        i += 2;
    }
    args
}
