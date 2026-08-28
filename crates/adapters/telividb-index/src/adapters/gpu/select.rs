//! Choosing the best `k` from a corpus-wide score row.
//!
//! **On why scoring everything is not post-filtering.** The matmul computes a
//! score for every row, including rows the caller may not see — but nothing
//! hidden ever reaches the selection step: excluded rows are dropped *before*
//! the top-k pass, so the `k` returned are drawn only from visible rows. That
//! is the property invariant 15 actually requires. The leak it forbids is
//! selecting `k` first and discarding afterwards, which returns fewer than `k`
//! and thereby reveals how many rows were hidden and where they ranked.
//!
//! **Visibility is resolved once per batch, not once per row per query.** The
//! predicate is called `rows` times and its answers kept, rather than
//! `queries * rows` times — at a batch of 32 over a million rows that is one
//! million calls instead of thirty-two million. This is the shape rule 36
//! already asks for ("policy is resolved once per query... compiles to a
//! bitmap"); applying it here costs one pass and one byte per row.
//!
//! It is also what makes selection parallel at all: a `&dyn Fn` is not `Sync`
//! and could not be shared across threads, while the mask of its answers is
//! just data.
//!
//! # Two axes of parallelism, and why the choice is measured
//!
//! Selection is ~83% of a batched device query — the matmul is one contiguous
//! pass at high arithmetic intensity, while this walks every score through a
//! branchy bounded heap. There are two independent ways to split it:
//!
//! - **across queries**, each with its own heap. Free and obviously correct,
//!   but a batch of one has nothing to split.
//! - **across row ranges** within one query, merging the partial heaps. This is
//!   what a single interactive query needs, since it is the only axis it has.
//!
//! Whichever axis has enough work to fill the machine is the one used. Doing
//! both at once would oversubscribe the pool and land the chunk merges behind
//! each other for no gain.

use super::corpus::DeviceCorpus;
use super::scan::Range;
use super::scored::Scored;
use crate::domain::{Candidate, TopK};
use std::borrow::Cow;
use telividb_core::Ordinal;

/// Rows below which splitting a query costs more than the scan it saves.
///
/// A chunk boundary costs a heap, a merge and a task hand-off; under roughly
/// this many rows that exceeds the scan itself.
#[cfg(feature = "parallel")]
const MIN_CHUNK: usize = 65_536;

impl DeviceCorpus {
    /// Which rows may be returned at all, presence and visibility combined.
    ///
    /// Borrows the presence bitmap when there is no predicate, so an unfiltered
    /// search allocates nothing.
    fn admissible<'a>(&'a self, allowed: Option<&dyn Fn(Ordinal) -> bool>) -> Cow<'a, [bool]> {
        match allowed {
            None => Cow::Borrowed(&self.present),
            Some(is_allowed) => Cow::Owned(
                self.present
                    .iter()
                    .enumerate()
                    .map(|(row, present)| *present && is_allowed(Ordinal::from_row(row as u32)))
                    .collect(),
            ),
        }
    }
}

impl Scored {
    /// The best `k` visible rows for query `q`.
    ///
    /// Splits the row range across threads, because one query cannot be split
    /// any other way and this is the path an interactive caller takes.
    pub(super) fn best(
        &self,
        corpus: &DeviceCorpus,
        q: usize,
        k: usize,
        allowed: Option<&dyn Fn(Ordinal) -> bool>,
    ) -> Vec<Candidate> {
        self.best_split(corpus, q, k, &corpus.admissible(allowed))
    }

    /// The best `k` for every query in the batch, in input order.
    pub(super) fn best_of_each(
        &self,
        corpus: &DeviceCorpus,
        queries: usize,
        k: usize,
        allowed: Option<&dyn Fn(Ordinal) -> bool>,
    ) -> Vec<Vec<Candidate>> {
        let admissible = corpus.admissible(allowed);

        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            // Enough queries to fill the pool: split across them and scan each
            // whole, which needs no merge at all.
            if queries >= rayon::current_num_threads() {
                return (0..queries)
                    .into_par_iter()
                    .map(|q| self.whole(corpus, q, k, &admissible))
                    .collect();
            }
        }

        (0..queries)
            .map(|q| self.best_split(corpus, q, k, &admissible))
            .collect()
    }

    /// One query, scanned as a single range.
    fn whole(
        &self,
        corpus: &DeviceCorpus,
        q: usize,
        k: usize,
        admissible: &[bool],
    ) -> Vec<Candidate> {
        let mut best = TopK::new(k, corpus.metric.higher_is_nearer());
        self.range_of(corpus, q, 0, self.dots(q).len(), admissible)
            .scan_into(&mut best);
        best.into_sorted()
    }

    /// One query, scanned as parallel row chunks and merged.
    ///
    /// **The merge is exactly equivalent to a serial scan**, not merely close.
    /// `TopK` orders candidates by `(score, ordinal)`, which is a *total* order
    /// — no two rows can tie, since ordinals are unique. The best `k` under a
    /// total order is therefore a single well-defined set, independent of the
    /// order candidates are offered in. Chunking changes only that order, so it
    /// cannot change the answer, and `into_sorted` breaks ties the same way at
    /// both levels.
    fn best_split(
        &self,
        corpus: &DeviceCorpus,
        q: usize,
        k: usize,
        admissible: &[bool],
    ) -> Vec<Candidate> {
        let rows = self.dots(q).len();

        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            let chunk = rows.div_ceil(rayon::current_num_threads()).max(MIN_CHUNK);
            if chunk < rows {
                let partials: Vec<Vec<Candidate>> = (0..rows)
                    .into_par_iter()
                    .step_by(chunk)
                    .map(|start| {
                        let mut best = TopK::new(k, corpus.metric.higher_is_nearer());
                        self.range_of(corpus, q, start, (start + chunk).min(rows), admissible)
                            .scan_into(&mut best);
                        best.into_sorted()
                    })
                    .collect();

                let mut best = TopK::new(k, corpus.metric.higher_is_nearer());
                for candidate in partials.into_iter().flatten() {
                    best.offer(candidate);
                }
                return best.into_sorted();
            }
        }

        self.whole(corpus, q, k, admissible)
    }

    /// The rows `start..end` of query `q`, ready to scan.
    fn range_of<'a>(
        &'a self,
        corpus: &'a DeviceCorpus,
        q: usize,
        start: usize,
        end: usize,
        admissible: &'a [bool],
    ) -> Range<'a> {
        Range {
            metric: corpus.metric,
            first_row: start,
            dots: &self.dots(q)[start..end],
            // Empty for metrics that never read it, so there is nothing to slice.
            row_norms: match corpus.row_norms.is_empty() {
                true => &[],
                false => &corpus.row_norms[start..end],
            },
            admissible: &admissible[start..end],
            query_norm: self.query_norm(q),
        }
    }
}
