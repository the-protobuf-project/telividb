//! Scoring a batch of queries against a resident corpus.
//!
//! One graph, one dispatch, for the whole batch. That is not a convenience —
//! ggml builds a graph and computes it in a single call, so a per-query
//! dispatch would pay the graph and submission cost once per query while
//! reading the entire corpus each time. Batching reads the corpus **once** for
//! every query in the call, which measured 5.5x per query at a batch of 32.
//!
//! # Why `mul_mat` gives what we want
//!
//! `ggml_mul_mat(a, b)` treats `ne[0]` as the shared dimension and produces
//! `result[i, j] = dot(a[.., i], b[.., j])`. With the corpus as `(dim, rows)`
//! and queries as `(dim, n)`, the result is `(rows, n)` — one column of scores
//! per query, contiguous in memory, which is exactly the shape selection wants.

use crate::corpus::Corpus;
use crate::error::{Error, Result};
use crate::graph::GraphRun;

/// Scores for a batch: `queries` rows of `corpus.rows()` scores each.
///
/// A value rather than a bare `Vec` so the shape travels with the numbers —
/// a flat vector of `n * rows` floats is trivially indexed wrongly, and the
/// mistake would rank every query against the wrong row.
#[derive(Debug, Clone)]
pub struct Scores {
    values: Vec<f32>,
    queries: usize,
    rows: usize,
}

impl Scores {
    /// Wrap a computed buffer with the shape that makes it readable.
    pub(crate) fn new(values: Vec<f32>, queries: usize, rows: usize) -> Self {
        Self {
            values,
            queries,
            rows,
        }
    }

    /// Scores for `queries` queries against no rows at all.
    ///
    /// A corpus with no rows cannot be uploaded — `ggml` has no zero-element
    /// tensor — so an empty corpus has nothing to call [`Corpus::score_batch`]
    /// on. This is the shape that call's result would have had, so a caller
    /// iterating per query needs no special case of its own.
    pub fn empty(queries: usize) -> Self {
        Self::new(Vec::new(), queries, 0)
    }

    /// Scores for one query, in row order.
    pub fn row(&self, query: usize) -> Option<&[f32]> {
        let start = query.checked_mul(self.rows)?;
        self.values.get(start..start + self.rows)
    }

    /// How many queries were scored.
    pub fn queries(&self) -> usize {
        self.queries
    }

    /// How many rows each query was scored against.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Every score, query-major.
    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }
}

impl Corpus {
    /// Score `queries` against every resident vector.
    ///
    /// `queries` is `count * dim` floats, each query contiguous. The result is
    /// the **inner product**; turning that into a metric — cosine by
    /// normalising at ingest, or L2 by the norm expansion — belongs to the
    /// layer above, which knows what the field was declared as.
    pub fn score_batch(&self, queries: &[f32], count: usize) -> Result<Scores> {
        if count == 0 {
            return Ok(Scores {
                values: Vec::new(),
                queries: 0,
                rows: self.rows(),
            });
        }
        if queries.len() != count * self.dim() {
            return Err(Error::ShapeMismatch {
                expected: format!("{} floats", count * self.dim()),
                actual: format!("{}", queries.len()),
            });
        }

        let graph = GraphRun::new(self, queries, count)?;
        graph.compute()
    }
}

#[cfg(test)]
#[path = "score_test.rs"]
mod tests;
