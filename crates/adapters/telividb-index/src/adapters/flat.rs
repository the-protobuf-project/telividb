//! Exhaustive search — the ground truth every approximate index is measured
//! against.
//!
//! Flat is not a placeholder. Recall for HNSW and IVF-PQ is defined as
//! agreement with this implementation, so it must stay obviously correct in
//! preference to being fast. See CLAUDE.md invariant 8.

use crate::domain::Candidate;
use crate::ports::{VectorIndex, VectorStore};
use std::time::Instant;
use telividb_core::{Ordinal, Result};
use telividb_telemetry::{Meter, fields, logger, metrics_names, redact};

/// Brute-force scan over every row in the store.
#[derive(Debug, Default, Clone)]
pub struct FlatIndex {
    /// Where search measurements go.
    ///
    /// On the index rather than in the search signature because `search` takes
    /// `&self` and runs concurrently on a shared index — there is nowhere in
    /// that call to pass `&mut` state. Disabled by default, so constructing an
    /// index needs no pipeline and no runtime.
    meter: Meter,
}

impl FlatIndex {
    /// A flat index. Holds no structure of its own beyond where to report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record search measurements through `meter`.
    pub fn with_meter(mut self, meter: Meter) -> Self {
        self.meter = meter;
        self
    }
}

impl VectorIndex for FlatIndex {
    fn kind(&self) -> &'static str {
        "flat"
    }

    fn search(
        &self,
        store: &dyn VectorStore,
        query: &[f32],
        k: usize,
        allowed: Option<&dyn Fn(Ordinal) -> bool>,
    ) -> Result<Vec<Candidate>> {
        let dim = store.dim().get();
        if query.len() != dim {
            return Err(telividb_core::Error::DimMismatch {
                expected: dim,
                actual: query.len(),
            });
        }
        if k == 0 {
            return Ok(Vec::new());
        }

        // One record on exit, not per-candidate instrumentation: the scan below
        // runs once per stored vector, and emitting there would cost more than
        // the distance computation it measured.
        let started = Instant::now();

        let metric = store.metric();
        let mut scored: Vec<Candidate> = Vec::new();
        let mut visited = 0u64;

        for row in 0..store.len() {
            let ordinal = Ordinal::from_row(row as u32);

            // The visibility bitmap is consulted *during* the scan, never after.
            // Filtering results afterwards would leak how many rows were hidden
            // and where they ranked — ARCHITECTURE.md §6.
            if let Some(is_allowed) = allowed
                && !is_allowed(ordinal)
            {
                continue;
            }

            // Absent is normal in a multimodal collection: a text-only point
            // has no image vector. ARCHITECTURE.md §4.1.
            let Some(candidate) = store.get(ordinal) else {
                continue;
            };

            let score = telividb_distance::score(metric, query, candidate);
            scored.push(Candidate::new(ordinal, score));
            visited += 1;
        }

        sort_best_first(&mut scored, k, metric.higher_is_nearer());
        scored.truncate(k);

        let elapsed = started.elapsed().as_secs_f64();
        // The stack's metrics take no attributes, so the index kind travels on
        // the log record rather than as a metric dimension. A dashboard that
        // wants flat-versus-hnsw latency reads it from there.
        self.meter
            .histogram(metrics_names::SEARCH_DURATION, elapsed);
        self.meter
            .histogram(metrics_names::SEARCH_CANDIDATES, visited as f64);
        self.meter
            .histogram(metrics_names::SEARCH_RESULTS, scored.len() as f64);

        logger::debug!("search complete").with_data(&serde_json::json!({
            fields::INDEX_KIND: self.kind(),
            fields::K: k,
            fields::DIM: dim,
            fields::FILTERED: allowed.is_some(),
            // Shape only. A query vector must never be emitted: it can be
            // inverted toward its source text, and logs are read by people who
            // were never granted `read_vector`.
            fields::QUERY: redact::vector_shape(query),
            fields::CANDIDATES_VISITED: visited,
            fields::RESULTS_RETURNED: scored.len(),
            fields::DURATION_SECONDS: elapsed,
        }));
        Ok(scored)
    }
}

/// Partition so the best `k` are in front, then order just those.
///
/// `select_nth_unstable_by` keeps this O(n) rather than O(n log n); only the
/// retained prefix is sorted.
pub(crate) fn sort_best_first(scored: &mut [Candidate], k: usize, higher_is_nearer: bool) {
    let better = |a: &Candidate, b: &Candidate| {
        // NaN scores would make this ordering inconsistent; they are rejected at
        // ingest, so `total_cmp` here is a defensive tie-break, not a policy.
        if higher_is_nearer {
            b.score.total_cmp(&a.score)
        } else {
            a.score.total_cmp(&b.score)
        }
    };

    if k < scored.len() {
        scored.select_nth_unstable_by(k, better);
        scored[..k].sort_unstable_by(better);
    } else {
        scored.sort_unstable_by(better);
    }
}

#[cfg(test)]
#[path = "flat_test.rs"]
mod tests;
