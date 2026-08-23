//! Measuring an approximate index against exhaustive search.
//!
//! Recall is the only number that makes an ANN change meaningful. A graph that
//! answers in a microsecond and returns the wrong neighbours is not fast, it is
//! broken — so every change to an approximate index reports recall@k against
//! [`crate::FlatIndex`] on a fixed dataset, or it is not a result.
//!
//! Deliberately compares **ordinals, not scores**. Two vectors at identical
//! distance are interchangeable to a caller but distinguishable here, so
//! score-based comparison reports spurious misses on tied data.

use crate::domain::Candidate;
use std::collections::HashSet;

/// Fraction of the true nearest neighbours that an approximate result found.
///
/// `1.0` means every true neighbour was returned; `0.0` means none were.
/// Normalised by the number of *truth* items, not by `k`: if exhaustive search
/// found only three neighbours because the corpus holds three rows, returning
/// all three is perfect recall rather than 0.3.
pub fn recall_at_k(approximate: &[Candidate], truth: &[Candidate], k: usize) -> f64 {
    let truth: HashSet<u32> = truth.iter().take(k).map(|c| c.ordinal.row()).collect();
    if truth.is_empty() {
        return 1.0;
    }
    let found = approximate
        .iter()
        .take(k)
        .filter(|c| truth.contains(&c.ordinal.row()))
        .count();
    found as f64 / truth.len() as f64
}

/// Recall averaged over a set of queries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecallReport {
    /// Number of queries this report averages over.
    pub queries: usize,
    /// The `k` recall was measured at.
    pub k: usize,
    /// Mean recall across every query.
    pub mean: f64,
    /// Worst single query. Mean recall hides a query class that fails
    /// completely, and that class is usually the one users notice.
    pub worst: f64,
}

impl RecallReport {
    /// Summarise per-query recall values.
    pub fn of(per_query: &[f64], k: usize) -> Self {
        if per_query.is_empty() {
            return Self {
                queries: 0,
                k,
                mean: 1.0,
                worst: 1.0,
            };
        }
        let sum: f64 = per_query.iter().sum();
        let worst = per_query.iter().copied().fold(f64::INFINITY, f64::min);
        Self {
            queries: per_query.len(),
            k,
            mean: sum / per_query.len() as f64,
            worst,
        }
    }

    /// Whether mean recall clears `threshold`.
    pub fn meets(&self, threshold: f64) -> bool {
        self.mean >= threshold
    }
}

impl std::fmt::Display for RecallReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "recall@{}: mean {:.4}, worst {:.4}, over {} queries",
            self.k, self.mean, self.worst, self.queries
        )
    }
}

#[cfg(test)]
#[path = "recall_test.rs"]
mod tests;
