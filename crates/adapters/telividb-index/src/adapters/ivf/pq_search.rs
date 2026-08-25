//! Querying an IVF-PQ index.
//!
//! Split from `pq.rs` so that file is about *building* the index — training two
//! quantizers and encoding residuals — and this is about the query path, which
//! runs far more often and has different constraints.

use super::pq::IvfPqIndex;
use crate::domain::{Candidate, TopK};
use crate::ports::VectorIndex;
use std::time::Instant;
use telividb_core::{Metric, Ordinal, Result, VectorStore};
use telividb_distance::Scorer;
use telividb_distance::pq::CENTROIDS;
use telividb_telemetry::{fields, logger, metrics_names};

impl VectorIndex for IvfPqIndex {
    fn kind(&self) -> &'static str {
        "ivf-pq"
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
        if k == 0 || self.coarse.is_empty() {
            return Ok(Vec::new());
        }

        let started = Instant::now();
        let metric = store.metric();
        let m = self.codebook.m();

        // Over-fetch on the approximate score, then re-rank exactly. The wider
        // net is what the rescore has to work with.
        let want = k.saturating_mul(self.rescore).max(k);
        let mut coarse_best = TopK::new(want, metric.higher_is_nearer());
        let mut visited = 0u64;

        for list in self.coarse.probe(query, metric, self.params.nprobe) {
            // The table is per *list*, because each list quantizes residuals
            // against its own centroid — one table for the whole query would
            // score every list's codes against the wrong origin.
            let residual = self.coarse.residual(query, list);
            let table = self.codebook.distance_table(&residual, metric)?;

            let entry = &self.lists[list];
            for (i, &row) in entry.rows.iter().enumerate() {
                let ordinal = Ordinal::from_row(row);
                if let Some(is_allowed) = allowed
                    && !is_allowed(ordinal)
                {
                    continue;
                }

                let codes = &entry.codes[i * m..(i + 1) * m];
                let score: f32 = codes
                    .iter()
                    .enumerate()
                    .map(|(sub, &code)| table[sub * CENTROIDS + code as usize])
                    .sum();
                coarse_best.offer(Candidate::new(ordinal, score));
                visited += 1;
            }
        }

        let found = rescore_exactly(store, query, metric, coarse_best.into_sorted(), k);

        let elapsed = started.elapsed().as_secs_f64();
        self.meter
            .histogram(metrics_names::SEARCH_DURATION, elapsed);
        logger::debug!("ivf-pq search").with_data(&serde_json::json!({
            fields::INDEX_KIND: "ivf-pq",
            fields::EF: self.params.nprobe,
            fields::CANDIDATES_VISITED: visited,
            fields::DURATION_SECONDS: elapsed,
        }));

        Ok(found)
    }
}

/// Re-rank approximate candidates against their true vectors.
///
/// A row whose vector is unavailable keeps its approximate score rather than
/// being dropped: absence is ordinary, and discarding it would return fewer
/// than `k` for a reason the caller cannot see.
fn rescore_exactly(
    store: &dyn VectorStore,
    query: &[f32],
    metric: Metric,
    candidates: Vec<Candidate>,
    k: usize,
) -> Vec<Candidate> {
    let mut best = TopK::new(k, metric.higher_is_nearer());
    for candidate in candidates {
        let score = match store.get(candidate.ordinal) {
            Some(vector) => metric.score(query, vector),
            None => candidate.score,
        };
        best.offer(Candidate::new(candidate.ordinal, score));
    }
    best.into_sorted()
}

#[cfg(test)]
#[path = "pq_search_test.rs"]
mod tests;

#[cfg(test)]
#[path = "pq_dials_test.rs"]
mod dial_tests;
