//! Rescoring quantized candidates at full precision.
//!
//! The second half of two-tier search. The first half scans compressed codes
//! wide and cheap; this half takes those candidates and rescores them against
//! `raw.bin`, which is where the accuracy comes back.
//!
//! Why it works: quantization perturbs scores by a bounded amount, so it rarely
//! moves a true neighbour *out* of a wide candidate set — but it reorders freely
//! *within* one. Over-fetching absorbs the first effect and reranking fixes the
//! second, which is why the pair recovers most of the recall the compression
//! cost while touching a fraction of the full-precision data.

use crate::domain::Candidate;
use episteme_core::{Metric, Ordinal, VectorStore};

/// How many candidates to pull from the quantized scan for a request of `k`.
///
/// The multiplier is the whole tuning surface: too small and true neighbours
/// never enter the candidate set, so reranking cannot recover them; too large
/// and the rerank costs as much as scanning at full precision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverFetch {
    pub multiplier: f32,
    pub minimum: usize,
}

impl Default for OverFetch {
    fn default() -> Self {
        // Four is the usual starting point for int8. Binary codes are far
        // coarser and want considerably more.
        Self {
            multiplier: 4.0,
            minimum: 32,
        }
    }
}

impl OverFetch {
    /// Candidates to request from the coarse scan for a top-`k` query.
    pub fn candidates_for(&self, k: usize) -> usize {
        ((k as f32 * self.multiplier).ceil() as usize)
            .max(self.minimum)
            .max(k)
    }
}

/// Rescore `candidates` against full-precision vectors and keep the best `k`.
///
/// Candidates whose rows are absent from `raw` are dropped rather than kept at
/// their approximate score: a row with no vector for this field cannot be
/// ranked, and carrying it forward on a coarse score would rank a non-answer.
pub fn rerank(
    raw: &dyn VectorStore,
    query: &[f32],
    candidates: &[Candidate],
    k: usize,
) -> Vec<Candidate> {
    let metric = raw.metric();
    let mut rescored: Vec<Candidate> = candidates
        .iter()
        .filter_map(|c| {
            raw.get(c.ordinal)
                .map(|v| Candidate::new(c.ordinal, episteme_distance::score(metric, query, v)))
        })
        .collect();

    sort_best_first(&mut rescored, metric);
    rescored.truncate(k);
    rescored
}

/// How much reranking changed the answer.
///
/// Worth reporting rather than inferring: if reordering is near zero the
/// over-fetch is wasted work, and if it is near total the coarse tier is too
/// lossy to be pruning on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RerankStats {
    pub considered: usize,
    pub returned: usize,
    /// Positions whose occupant changed after rescoring.
    pub reordered: usize,
}

/// Rerank, and report what it changed.
pub fn rerank_measured(
    raw: &dyn VectorStore,
    query: &[f32],
    candidates: &[Candidate],
    k: usize,
) -> (Vec<Candidate>, RerankStats) {
    let before: Vec<Ordinal> = candidates.iter().take(k).map(|c| c.ordinal).collect();
    let after = rerank(raw, query, candidates, k);

    let reordered = after
        .iter()
        .enumerate()
        .filter(|(i, c)| before.get(*i) != Some(&c.ordinal))
        .count();

    let stats = RerankStats {
        considered: candidates.len(),
        returned: after.len(),
        reordered,
    };
    (after, stats)
}

fn sort_best_first(candidates: &mut [Candidate], metric: Metric) {
    if metric.higher_is_nearer() {
        candidates.sort_unstable_by(|a, b| b.score.total_cmp(&a.score));
    } else {
        candidates.sort_unstable_by(|a, b| a.score.total_cmp(&b.score));
    }
}

#[cfg(test)]
#[path = "rerank_test.rs"]
mod tests;
