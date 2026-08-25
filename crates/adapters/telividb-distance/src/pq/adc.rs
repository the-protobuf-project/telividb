//! Asymmetric distance computation: scoring a code without decoding it.
//!
//! Split from `codebook.rs` because training and encoding are about building
//! the codebook, and this is about *using* one at query time — the hot path,
//! and the reason product quantization is fast rather than merely small.

use super::codebook::{CENTROIDS, PqCodebook};
use telividb_core::{Error, Metric, Result};

impl PqCodebook {
    /// Partial scores for every `(subspace, centroid)` pair against `query`.
    ///
    /// **Asymmetric distance computation.** The query stays full precision and
    /// only the stored rows are quantized, which is why this is more accurate
    /// than comparing two codes. Scoring a row afterwards costs `m` lookups and
    /// `m` adds — no multiplies and no vector reconstruction — which is the
    /// entire reason product quantization is fast rather than merely small.
    ///
    /// Laid out `m * CENTROIDS` so a row's score is
    /// `sum(table[sub * CENTROIDS + code[sub]])`.
    ///
    /// Summing partial scores is exact for both dot and squared L2, because
    /// both decompose over disjoint subspaces. It would *not* be for a metric
    /// that does not, which is why this takes the metric rather than assuming.
    pub fn distance_table(&self, query: &[f32], metric: Metric) -> Result<Vec<f32>> {
        if query.len() != self.dim {
            return Err(Error::DimMismatch {
                expected: self.dim,
                actual: query.len(),
            });
        }

        let mut table = Vec::with_capacity(self.m * CENTROIDS);
        for sub in 0..self.m {
            let start = sub * self.sub_dim;
            let part = &query[start..start + self.sub_dim];
            for index in 0..CENTROIDS {
                table.push(partial(metric, part, self.centroid(sub, index)));
            }
        }
        Ok(table)
    }
}

/// One subspace's contribution, under the metric that will sum them.
fn partial(metric: Metric, query: &[f32], centroid: &[f32]) -> f32 {
    match metric {
        Metric::Dot | Metric::Cosine => query.iter().zip(centroid).map(|(a, b)| a * b).sum(),
        Metric::L2 => crate::l2_squared(query, centroid),
    }
}
