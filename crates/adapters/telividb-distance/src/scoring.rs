//! Scoring, as a method on the metric.
//!
//! `Metric` is a domain type defined in `telividb-core`, so an inherent `impl`
//! here is forbidden by the orphan rule. A trait is how behaviour attaches to a
//! type another crate owns while keeping the receiver form — `metric.score(q, c)`
//! rather than `score(metric, q, c)`.

use crate::ops::VectorOps;
use telividb_core::Metric;

/// Comparing a query against a stored vector.
pub trait Scorer {
    /// How near `candidate` is to `query`, under this metric.
    ///
    /// The direction differs by metric and is not encoded in the number:
    /// [`Metric::higher_is_nearer`] says which way to sort, and every selection
    /// path must consult it rather than assume.
    fn score(&self, query: &[f32], candidate: &[f32]) -> f32;
}

impl Scorer for Metric {
    fn score(&self, query: &[f32], candidate: &[f32]) -> f32 {
        match self {
            // Cosine is stored normalised and scored as a dot product — the
            // division happens once at ingest rather than per comparison. A
            // vector that reached here un-normalised scores by magnitude as
            // well as direction, which looks like a quality problem rather
            // than a bug, so normalisation is an ingest invariant.
            Metric::Dot | Metric::Cosine => query.dot(candidate),
            Metric::L2 => query.l2_squared(candidate),
        }
    }
}

#[cfg(test)]
#[path = "scoring_test.rs"]
mod tests;
