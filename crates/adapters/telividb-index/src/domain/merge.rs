//! Combining buffer and segment results into one ranked answer.
//!
//! A query touches two kinds of source: the **unsealed buffer**, scanned
//! exhaustively, and any number of **sealed segments**, searched through their
//! indexes. Top-k selection happens across the union — never per source, which
//! would return `k` results from each and then need reconciling anyway.
//!
//! The provenance split is not bookkeeping. The buffer scan is exact, so its
//! hits can only improve the answer; if a recall measurement counts them
//! alongside index hits it is measuring a mixture and reporting it as index
//! quality. Every merge therefore reports where its results came from.

use crate::domain::Candidate;
use telividb_core::Ordinal;

/// Where a hit came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The unsealed write buffer. Scanned exhaustively, so **exact**.
    Buffer,
    /// A sealed segment, reached through its index. Approximate for HNSW and
    /// IVF; exact only for a flat index.
    Sealed(
        /// Identifier of the segment the hit came from.
        u64,
    ),
}

impl Source {
    /// Whether results from this source are exhaustive.
    ///
    /// Recall is only meaningful over the approximate portion of an answer.
    pub fn is_exact(self) -> bool {
        matches!(self, Source::Buffer)
    }
}

/// One result, still carrying the source that produced it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// Which store produced this hit, and therefore whether it is exact.
    pub source: Source,
    /// Row within its source. Meaningless outside the segment or buffer
    /// named by `source`.
    pub ordinal: Ordinal,
    /// Score on the field's metric scale.
    pub score: f32,
}

/// What a merge drew on, so recall can be attributed correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MergeStats {
    /// Returned hits that came from the exact buffer scan.
    pub buffer_hits: usize,
    /// Returned hits that came from a sealed segment index.
    pub sealed_hits: usize,
    /// Sources consulted, whether or not they contributed.
    pub sources_searched: usize,
}

impl MergeStats {
    /// Whether this answer is entirely exact, and so not a recall sample.
    ///
    /// A result set drawn only from the buffer says nothing about index
    /// quality; counting it would drag measured recall toward 1.0 and hide a
    /// genuinely degraded index.
    pub fn is_fully_exact(&self) -> bool {
        self.sealed_hits == 0
    }
}

/// A merged answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Merged {
    /// Results, best first.
    pub hits: Vec<Hit>,
    /// Where those results came from.
    pub stats: MergeStats,
}

/// Select the best `k` across every source.
///
/// `higher_is_nearer` follows the field's metric: dot and cosine rank
/// descending, L2 ascending. Getting it wrong returns the *worst* `k` matches,
/// which is why it is a parameter rather than an assumption.
pub fn merge_top_k(
    sources: &[(Source, Vec<Candidate>)],
    k: usize,
    higher_is_nearer: bool,
) -> Merged {
    let mut stats = MergeStats {
        sources_searched: sources.len(),
        ..Default::default()
    };

    if k == 0 {
        return Merged {
            hits: Vec::new(),
            stats,
        };
    }

    let mut all: Vec<Hit> = sources
        .iter()
        .flat_map(|(source, candidates)| {
            candidates.iter().map(move |c| Hit {
                source: *source,
                ordinal: c.ordinal,
                score: c.score,
            })
        })
        .collect();

    let better = |a: &Hit, b: &Hit| {
        if higher_is_nearer {
            b.score.total_cmp(&a.score)
        } else {
            a.score.total_cmp(&b.score)
        }
    };

    if k < all.len() {
        all.select_nth_unstable_by(k, better);
        all.truncate(k);
    }
    all.sort_unstable_by(better);

    for hit in &all {
        match hit.source {
            Source::Buffer => stats.buffer_hits += 1,
            Source::Sealed(_) => stats.sealed_hits += 1,
        }
    }

    Merged { hits: all, stats }
}

#[cfg(test)]
#[path = "merge_test.rs"]
mod tests;
