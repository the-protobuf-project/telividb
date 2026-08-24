//! Composing a coarse scan with an exact rerank.
//!
//! The search path the storage design exists for: scan wide and cheap over a
//! [`ScanTier`], then rescore the survivors at full precision from a
//! [`VectorStore`]. Neither side knows about the other — the tier does not know
//! it will be reranked, and the store does not know it was preceded by a scan.
//!
//! Why the composition lives here rather than inside a store: whether to use
//! the coarse tier at all is a **planning decision**, not a storage property. A
//! selective filter may leave so few rows that scanning exactly is cheaper; a
//! small field may have no tier; a caller wanting exact results can skip it.
//! Keeping the choice in the search path means all of those are the same code
//! with different arguments.

use crate::domain::{Candidate, OverFetch, RerankStats, TopK};
use episteme_core::{Ordinal, Result, ScanTier, VectorStore};

/// What a two-tier search did, for query explain and for tuning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TwoTierStats {
    /// Rows the coarse tier scored.
    pub scanned: usize,
    /// Candidates carried into the rerank.
    pub candidates: usize,
    /// Results returned after truncation to `k`.
    pub returned: usize,
    /// Positions the rerank changed.
    pub reordered: usize,
}

impl TwoTierStats {
    /// Fraction of the corpus that reached full precision.
    ///
    /// The number that says whether two-tier is earning anything: near 1.0 and
    /// the coarse pass is pure overhead.
    pub fn rerank_fraction(&self) -> f64 {
        if self.scanned == 0 {
            return 0.0;
        }
        self.candidates as f64 / self.scanned as f64
    }
}

/// Scan `tier` for candidates, then rerank them against `exact`.
///
/// `allowed` is consulted **during the scan**, never afterwards: filtering
/// results after the fact leaks how many rows were hidden and where they
/// ranked.
pub fn search(
    tier: &dyn ScanTier,
    exact: &dyn VectorStore,
    query: &[f32],
    k: usize,
    over_fetch: OverFetch,
    allowed: Option<&dyn Fn(Ordinal) -> bool>,
) -> Result<(Vec<Candidate>, TwoTierStats)> {
    if k == 0 {
        return Ok((Vec::new(), TwoTierStats::default()));
    }

    // Checked here rather than left to each tier: the exact store always knows
    // the field's width, and a tier that forgot to validate would otherwise
    // score against a truncated query and return confident nonsense.
    let dim = exact.dim().get();
    if query.len() != dim {
        return Err(episteme_core::Error::DimMismatch {
            expected: dim,
            actual: query.len(),
        });
    }

    let metric = exact.metric();
    let prepared = tier.prepare(query, metric)?;
    let want = over_fetch.candidates_for(k);

    // Bounded rather than collect-then-sort. This loop runs once per row in
    // the field, and the pass exists to be the cheap one: materialising every
    // score and sorting the lot to keep `want` of them is an O(n) allocation
    // and an O(n log n) sort per query.
    let mut best = TopK::new(want, metric.higher_is_nearer());
    let mut scanned = 0usize;

    for row in 0..tier.len() {
        let ordinal = Ordinal::from_row(row as u32);
        if let Some(is_allowed) = allowed
            && !is_allowed(ordinal)
        {
            continue;
        }
        let Some(score) = tier.score(&prepared, ordinal) else {
            continue;
        };
        scanned += 1;
        best.offer(Candidate::new(ordinal, score));
    }

    let coarse = best.into_sorted();

    let (hits, rerank_stats) = crate::domain::rerank_measured(exact, query, &coarse, k);
    let stats = TwoTierStats {
        scanned,
        candidates: coarse.len(),
        returned: hits.len(),
        reordered: rerank_stats.reordered,
    };
    Ok((hits, stats))
}

/// Rerank an existing candidate set, reporting what changed.
///
/// The seam an approximate index plugs into: HNSW produces candidates from a
/// quantized tier, and this rescores them without either side knowing about
/// the other.
pub fn rerank_candidates(
    exact: &dyn VectorStore,
    query: &[f32],
    candidates: &[Candidate],
    k: usize,
) -> (Vec<Candidate>, RerankStats) {
    crate::domain::rerank_measured(exact, query, candidates, k)
}

#[cfg(test)]
#[path = "two_tier_test.rs"]
mod tests;
