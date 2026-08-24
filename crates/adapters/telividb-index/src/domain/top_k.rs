//! Keeping the best `k` candidates without holding the rest.

use crate::domain::Candidate;
use std::collections::BinaryHeap;

/// A bounded collector for the best `k` candidates seen.
///
/// # Why not collect and sort
///
/// The coarse scan visits every row in the field. Pushing each one into a
/// `Vec` and sorting at the end is an O(n) allocation and an O(n log n) sort
/// per query — on the pass whose entire purpose is being the cheap one. For a
/// ten-million-row field returning `k = 10`, that sorts ten million candidates
/// to discard all but ten of them.
///
/// A bounded heap holds `k` and compares each row against the worst of them:
/// O(n log k) time and O(k) space, with the log k paid only when a row is good
/// enough to displace something.
#[derive(Debug)]
pub struct TopK {
    /// The best candidates so far, worst-first so the loser pops cheaply.
    heap: BinaryHeap<Worst>,
    /// How many to keep.
    k: usize,
    /// Whether a larger score is a better one for this metric.
    higher_is_nearer: bool,
}

impl TopK {
    /// A collector keeping the best `k` under `higher_is_nearer`.
    pub fn new(k: usize, higher_is_nearer: bool) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(k.saturating_add(1).min(4096)),
            k,
            higher_is_nearer,
        }
    }

    /// Offer a candidate, keeping it only if it beats the current worst.
    pub fn offer(&mut self, candidate: Candidate) {
        if self.k == 0 {
            return;
        }
        self.heap.push(Worst {
            candidate,
            higher_is_nearer: self.higher_is_nearer,
        });
        if self.heap.len() > self.k {
            // Pops the worst, because `Worst` inverts the ordering.
            self.heap.pop();
        }
    }

    /// The kept candidates, best first.
    pub fn into_sorted(self) -> Vec<Candidate> {
        let higher_is_nearer = self.higher_is_nearer;
        let mut out: Vec<Candidate> = self.heap.into_iter().map(|w| w.candidate).collect();
        if higher_is_nearer {
            out.sort_unstable_by(|a, b| b.score.total_cmp(&a.score));
        } else {
            out.sort_unstable_by(|a, b| a.score.total_cmp(&b.score));
        }
        out
    }
}

/// A candidate ordered so the *worst* compares greatest.
///
/// `BinaryHeap` is a max-heap, so inverting the comparison puts the candidate
/// most deserving of eviction at the top — which is the one `offer` pops.
#[derive(Debug)]
struct Worst {
    /// The candidate being ordered.
    candidate: Candidate,
    /// Which direction counts as better.
    higher_is_nearer: bool,
}

impl Ord for Worst {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // `total_cmp` rather than `partial_cmp`: a NaN score would make the
        // ordering inconsistent and `BinaryHeap` relies on it being a total
        // order. NaN is rejected at ingest, so this is a defensive tie-break
        // rather than a policy — but an inconsistent `Ord` is a logic bug that
        // surfaces as silently wrong results, not a panic.
        let ordering = self.candidate.score.total_cmp(&other.candidate.score);
        if self.higher_is_nearer {
            // Lower score is worse, so it must compare greater.
            ordering.reverse()
        } else {
            ordering
        }
        // Tie-break on ordinal so eviction is deterministic: two rows with the
        // same score must not depend on heap internals for which survives.
        .then(
            self.candidate
                .ordinal
                .row()
                .cmp(&other.candidate.ordinal.row()),
        )
    }
}

impl PartialOrd for Worst {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Worst {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for Worst {}

#[cfg(test)]
#[path = "top_k_test.rs"]
mod tests;
