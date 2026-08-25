//! What the device returned, and how to read it.
//!
//! Split from `corpus.rs` because the two answer different questions: that file
//! is about vectors resident on a device, this is about the score matrix one
//! call over them produced. They also change for different reasons — a new
//! metric touches this, a new upload strategy touches that.

/// Device-computed inner products, plus what turns them into metric scores.
pub(super) struct Scored {
    /// One row of inner products per query.
    pub(super) scores: telividb_compute::Scores,
    /// `‖query‖²`, for the L2 expansion. Empty for other metrics.
    pub(super) query_norms: Vec<f32>,
}

impl Scored {
    /// Query `q`'s raw inner products, one per row.
    pub(super) fn dots(&self, q: usize) -> &[f32] {
        self.scores.row(q).unwrap_or(&[])
    }

    /// `‖query q‖²`, or zero where the metric does not need it.
    pub(super) fn query_norm(&self, q: usize) -> f32 {
        self.query_norms.get(q).copied().unwrap_or(0.0)
    }
}
