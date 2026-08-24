//! Scoring a whole corpus in one matmul, then selecting the best `k`.
//!
//! **On why scoring everything is not post-filtering.** The matmul computes a
//! score for every row, including rows the caller may not see — but nothing
//! hidden ever reaches the selection step: excluded rows are dropped *before*
//! the top-k pass, so the `k` returned are drawn only from visible rows. That
//! is the property invariant 15 actually requires. The leak it forbids is
//! selecting `k` first and discarding afterwards, which returns fewer than `k`
//! and thereby reveals how many rows were hidden and where they ranked.

use super::gguf::Corpus;
use crate::adapters::flat::sort_best_first;
use crate::domain::Candidate;
use candle_core::Tensor;
use telividb_core::{Metric, Ordinal, Result};

/// Score `query` against every row of `corpus` and return the best `k`.
pub(super) fn search(
    corpus: &Corpus,
    query: &[f32],
    k: usize,
    allowed: Option<&dyn Fn(Ordinal) -> bool>,
) -> Result<Vec<Candidate>> {
    let dim = corpus.dim.get();
    if query.len() != dim {
        return Err(telividb_core::Error::DimMismatch {
            expected: dim,
            actual: query.len(),
        });
    }
    if k == 0 || corpus.present.is_empty() {
        return Ok(Vec::new());
    }

    let scores = score_all(corpus, query)?;

    let mut scored: Vec<Candidate> = Vec::new();
    for (row, score) in scores.iter().enumerate() {
        // Absent for this field: the row holds zeros, and against a dot
        // product zero is a real score rather than a neutral one, so it must
        // be excluded rather than allowed to rank.
        if !corpus.present[row] {
            continue;
        }
        let ordinal = Ordinal::from_row(row as u32);
        if let Some(is_allowed) = allowed
            && !is_allowed(ordinal)
        {
            continue;
        }
        scored.push(Candidate::new(ordinal, *score));
    }

    sort_best_first(&mut scored, k, corpus.metric.higher_is_nearer());
    scored.truncate(k);
    Ok(scored)
}

/// Every row's score against `query`, in one device operation.
///
/// Cosine is stored normalised and scored as dot (the CLAUDE.md gotcha), so a
/// single `(1, dim) × (dim, rows)` product *is* the scoring function for both.
/// L2 would need the `‖a‖² − 2a·b + ‖b‖²` expansion with precomputed row
/// norms; until that exists it is refused rather than scored as a dot product,
/// which would return confidently wrong neighbours.
fn score_all(corpus: &Corpus, query: &[f32]) -> Result<Vec<f32>> {
    if corpus.metric == Metric::L2 {
        return Err(telividb_core::Error::GpuIndex {
            reason: "L2 is not yet implemented on the GPU index; \
                     use dot or cosine, or the CPU flat index"
                .to_owned(),
        });
    }

    let device = corpus.tensor.device();
    let query = Tensor::from_slice(query, (1, corpus.dim.get()), device).map_err(candle_err)?;

    // (1, dim) x (dim, rows) -> (1, rows)
    let scores = query
        .matmul(&corpus.tensor.t().map_err(candle_err)?)
        .map_err(candle_err)?;

    scores
        .flatten_all()
        .map_err(candle_err)?
        .to_vec1()
        .map_err(candle_err)
}

fn candle_err(e: candle_core::Error) -> telividb_core::Error {
    telividb_core::Error::GpuIndex {
        reason: e.to_string(),
    }
}

#[cfg(test)]
#[path = "search_test.rs"]
mod tests;
