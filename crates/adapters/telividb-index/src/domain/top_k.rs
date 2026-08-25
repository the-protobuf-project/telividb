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
    ///
    /// Once the heap is full, a losing candidate costs one comparison. That
    /// matters more than it looks: an exhaustive scan offers every row, so the
    /// overwhelming majority of calls lose, and paying a push and a pop for
    /// each of them was measurably the largest cost in a GPU query — 2.1 ms of
    /// a 5.7 ms query on a million-row corpus.
    ///
    /// A cached-threshold fast path was tried here and **measured as noise** on
    /// a million rows, so it was removed rather than kept on the theory that it
    /// ought to help: `BinaryHeap::peek` is already cheap and the branch
    /// predicts perfectly. Recorded so the next person does not re-derive it.
    pub fn offer(&mut self, candidate: Candidate) {
        if self.k == 0 {
            return;
        }
        let candidate = Worst {
            candidate,
            higher_is_nearer: self.higher_is_nearer,
        };

        if self.heap.len() == self.k {
            // `Worst` inverts the ordering, so "beats the worst kept" is
            // `candidate < worst`.
            match self.heap.peek() {
                Some(worst) if candidate < *worst => {
                    self.heap.pop();
                }
                _ => return,
            }
        }
        self.heap.push(candidate);
    }

    /// The kept candidates, best first.
    ///
    /// Ties break on the row, ascending — the same rule eviction uses. Without
    /// it the final order of equal-scoring rows depends on the heap's internal
    /// layout, so two indexes over the same data return the same *set* in
    /// different orders purely because one scanned sequentially and the other
    /// list by list. That is invisible until something compares them.
    pub fn into_sorted(self) -> Vec<Candidate> {
        let higher_is_nearer = self.higher_is_nearer;
        let mut out: Vec<Candidate> = self.heap.into_iter().map(|w| w.candidate).collect();
        out.sort_unstable_by(|a, b| {
            let by_score = match higher_is_nearer {
                true => b.score.total_cmp(&a.score),
                false => a.score.total_cmp(&b.score),
            };
            by_score.then_with(|| a.ordinal.row().cmp(&b.ordinal.row()))
        });
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
