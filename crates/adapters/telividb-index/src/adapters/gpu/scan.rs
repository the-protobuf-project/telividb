//! Scanning one contiguous stretch of rows into a bounded heap.
//!
//! Factored out of `select.rs` because it is the unit both selection strategies
//! share: a whole-corpus scan is one range, and a row-chunked scan is several
//! that merge. Having one implementation is what makes the two provably
//! equivalent rather than merely intended to be.

use super::metric::ScoreFromDot;
use crate::domain::{Candidate, TopK};
use telividb_core::{Metric, Ordinal};

/// A stretch of consecutive rows and everything needed to score them.
///
/// Carries `first_row` because a chunk's slices are zero-based while ordinals
/// are corpus-wide. Getting that offset wrong produces plausible results
/// pointing at the wrong rows, which is the failure mode least likely to be
/// noticed, so it is a field rather than an argument threaded through calls.
pub(super) struct Range<'a> {
    /// What a score means, and which direction is nearer.
    pub(super) metric: Metric,
    /// Corpus-wide ordinal of `dots[0]`.
    pub(super) first_row: usize,
    /// Inner products for these rows, one per row.
    pub(super) dots: &'a [f32],
    /// `‖row‖²` for these rows. Empty unless the metric needs it.
    pub(super) row_norms: &'a [f32],
    /// Whether each row may be returned — presence and visibility combined.
    pub(super) admissible: &'a [bool],
    /// `‖query‖²`, or zero where the metric does not need it.
    pub(super) query_norm: f32,
}

impl Range<'_> {
    /// Offer every admissible row in this range to `best`.
    ///
    /// **Split by metric outside the loop, not inside it.** This is the hottest
    /// loop in the system — a million iterations per query — and the metric is
    /// constant across every one of them. Hoisting the branch lets each arm
    /// fetch exactly the inputs it needs: the L2 arm walks `row_norms`
    /// alongside the scores, and the dot arm never touches it.
    ///
    /// Both arms still call [`ScoreFromDot::score_of`], so there remains one
    /// definition of what a score means; with the metric constant in each arm
    /// its match folds away entirely.
    pub(super) fn scan_into(&self, best: &mut TopK) {
        let metric = self.metric;
        let first = self.first_row;

        // Absent or hidden rows are skipped before `offer`. An absent row holds
        // zeros, and against a dot product zero is a real score rather than a
        // neutral one, so it would rank if it were not excluded here.
        match metric {
            Metric::L2 => {
                let rows = self
                    .dots
                    .iter()
                    .zip(self.row_norms)
                    .zip(self.admissible)
                    .enumerate();
                for (row, ((dot, row_norm), admit)) in rows {
                    if *admit {
                        let score = metric.score_of(*dot, *row_norm, self.query_norm);
                        best.offer(Candidate::new(
                            Ordinal::from_row((first + row) as u32),
                            score,
                        ));
                    }
                }
            }
            // Dot and cosine score as the product itself, so no norms are read.
            _ => {
                for (row, (dot, admit)) in self.dots.iter().zip(self.admissible).enumerate() {
                    if *admit {
                        let score = metric.score_of(*dot, 0.0, 0.0);
                        best.offer(Candidate::new(
                            Ordinal::from_row((first + row) as u32),
                            score,
                        ));
                    }
                }
            }
        }
    }
}
