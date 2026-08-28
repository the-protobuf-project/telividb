//! What an inner product means, per metric.
//!
//! The device computes one thing — `a·b` for every (query, row) pair — and
//! every metric this index supports is recoverable from it. Keeping that
//! conversion in one place is the point: two implementations of "what a score
//! means" (one for single queries, one for batches) would eventually disagree,
//! and the disagreement would show up as a ranking that is subtly wrong rather
//! than as a failure.
//!
//! A trait rather than an inherent method because [`Metric`] belongs to
//! `telividb-core`; the orphan rule puts inherent methods out of reach from
//! here, and a trait keeps the receiver form regardless.

use telividb_core::Metric;

/// Recovering a metric's score from an inner product.
pub(super) trait ScoreFromDot {
    /// The score for one (query, row) pair, given their inner product.
    ///
    /// `row_norm` and `query_norm` are `‖·‖²` and are consulted only where the
    /// metric needs them, so a caller scoring dot or cosine may pass zero
    /// rather than computing norms it will not use.
    fn score_of(self, dot: f32, row_norm: f32, query_norm: f32) -> f32;
}

impl ScoreFromDot for Metric {
    fn score_of(self, dot: f32, row_norm: f32, query_norm: f32) -> f32 {
        match self {
            // `‖a − b‖² = ‖a‖² − 2a·b + ‖b‖²`.
            //
            // **Squared, not the square root.** Ranking is identical either way
            // since the root is monotonic, and skipping it saves a pass over
            // every row — which is why `Metric::L2` is documented as squared
            // Euclidean throughout.
            //
            // `‖a‖²` is constant across rows and so cannot change the ordering,
            // but it is included anyway: the scores are returned to the caller,
            // and a distance that ranks correctly while being numerically wrong
            // is exactly the kind of thing that is trusted until someone uses it
            // for a threshold.
            Metric::L2 => query_norm - 2.0 * dot + row_norm,
            // Cosine is stored normalised and scored as dot (the CLAUDE.md
            // gotcha), so for both the product *is* the score.
            _ => dot,
        }
    }
}

#[cfg(test)]
#[path = "metric_test.rs"]
mod tests;
