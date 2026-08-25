//! Choosing the best `k` from a corpus-wide score row.
//!
//! **On why scoring everything is not post-filtering.** The matmul computes a
//! score for every row, including rows the caller may not see — but nothing
//! hidden ever reaches the selection step: excluded rows are dropped *before*
//! the top-k pass, so the `k` returned are drawn only from visible rows. That
//! is the property invariant 15 actually requires. The leak it forbids is
//! selecting `k` first and discarding afterwards, which returns fewer than `k`
//! and thereby reveals how many rows were hidden and where they ranked.

use super::corpus::{DeviceCorpus, Scored};
use crate::domain::{Candidate, TopK};
use telividb_core::Ordinal;

impl Scored {
    /// The best `k` visible rows for query `q`.
    ///
    /// Bounded rather than collect-then-sort. The device scores every row, and
    /// pushing all of them into a `Vec` to keep `k` cost more than the matmul
    /// itself: 2.1 ms of a 5.7 ms query on a million rows.
    pub(super) fn best(
        &self,
        corpus: &DeviceCorpus,
        q: usize,
        k: usize,
        allowed: Option<&dyn Fn(Ordinal) -> bool>,
    ) -> Vec<Candidate> {
        let mut best = TopK::new(k, corpus.metric.higher_is_nearer());
        for (row, score) in self.row(corpus, q).enumerate() {
            // Absent for this field: the row holds zeros, and against a dot
            // product zero is a real score rather than a neutral one, so it
            // must be excluded rather than allowed to rank.
            if !corpus.present[row] {
                continue;
            }
            let ordinal = Ordinal::from_row(row as u32);
            if let Some(is_allowed) = allowed
                && !is_allowed(ordinal)
            {
                continue;
            }
            best.offer(Candidate::new(ordinal, score));
        }
        best.into_sorted()
    }
}
