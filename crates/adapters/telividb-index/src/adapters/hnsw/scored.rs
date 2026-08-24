//! Ordering by distance, where lower is always nearer.
//!
//! HNSW is written against a *distance*, but the system's metrics disagree
//! about direction — dot and cosine rank descending, L2 ascending. Rather than
//! thread that difference through every comparison, scores are converted once
//! on entry so the algorithm only ever deals with "smaller is better".

use telividb_core::{Metric, Ordinal};

/// Convert a metric score into a distance where lower is nearer.
///
/// Negation rather than `1 - x` for the inner-product metrics: it preserves
/// ordering exactly and cannot lose precision for scores outside `[0, 1]`,
/// which unnormalised dot products routinely are.
pub fn to_distance(metric: Metric, score: f32) -> f32 {
    if metric.higher_is_nearer() {
        -score
    } else {
        score
    }
}

/// Convert a distance back into the metric's own score.
pub fn from_distance(metric: Metric, distance: f32) -> f32 {
    if metric.higher_is_nearer() {
        -distance
    } else {
        distance
    }
}

/// A node with its distance, ordered nearest-first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scored {
    pub distance: f32,
    pub ordinal: Ordinal,
}

impl Scored {
    /// Pair an ordinal with its distance from the query.
    pub fn new(distance: f32, ordinal: Ordinal) -> Self {
        Self { distance, ordinal }
    }
}

impl Eq for Scored {}

impl Ord for Scored {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // `total_cmp` keeps the ordering total even if a NaN slipped through,
        // which matters because `BinaryHeap` misbehaves badly on a partial
        // order — it does not merely return odd results, it can loop.
        self.distance
            .total_cmp(&other.distance)
            .then_with(|| self.ordinal.row().cmp(&other.ordinal.row()))
    }
}

impl PartialOrd for Scored {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// [`Scored`] with the ordering flipped, for use in a max-heap.
///
/// `BinaryHeap` is a max-heap, so keeping the *worst* candidate at the top —
/// which is what bounding a result set to `ef` requires — needs reversed
/// ordering rather than a second heap type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Farthest(pub Scored);

impl Ord for Farthest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Farthest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// [`Scored`] ordered nearest-first in a max-heap, i.e. a min-heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nearest(pub Scored);

impl Ord for Nearest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.cmp(&self.0)
    }
}

impl PartialOrd for Nearest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
#[path = "scored_test.rs"]
mod tests;
