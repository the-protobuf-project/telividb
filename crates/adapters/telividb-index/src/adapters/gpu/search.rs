//! The port implementation: one device call per query batch, then selection.
//!
//! Split from `mod.rs` so that file stays about what a `GpuFlatIndex` *is* —
//! what it owns, what it accounts for — while this is about what it does when
//! asked. The two change for different reasons.

use super::GpuFlatIndex;
use crate::domain::Candidate;
use crate::ports::{VectorIndex, VectorStore};
use std::time::Instant;
use telividb_core::{Ordinal, Result};
use telividb_telemetry::{fields, logger, metrics_names, redact};

/// Queries per device call.
///
/// From measurement rather than theory: throughput improves up to roughly this
/// size and then reverses as the returned score matrix grows — at 128 queries
/// over a million rows it is 512 MB to copy back, and the copy that was
/// negligible for one query dominates. A larger batch is split into chunks of
/// this size.
const MAX_BATCH: usize = 32;

impl VectorIndex for GpuFlatIndex {
    fn kind(&self) -> &'static str {
        "gpu-flat"
    }

    fn search_batch(
        &self,
        _store: &dyn VectorStore,
        queries: &[&[f32]],
        k: usize,
        allowed: Option<&dyn Fn(Ordinal) -> bool>,
    ) -> Result<Vec<Vec<Candidate>>> {
        for query in queries {
            self.check_dim(query)?;
        }
        if queries.is_empty() {
            return Ok(Vec::new());
        }
        if k == 0 || self.corpus.rows() == 0 {
            return Ok(vec![Vec::new(); queries.len()]);
        }

        // One device call per chunk: the corpus is read once for the whole
        // chunk rather than once per query, which is where the speed-up comes
        // from — 2.232 ms per query answered singly, 0.409 ms at a batch of 32.
        let mut out = Vec::with_capacity(queries.len());
        let mut on_device = 0.0;
        let mut on_host = 0.0;

        for chunk in queries.chunks(MAX_BATCH) {
            let flat: Vec<f32> = chunk.iter().flat_map(|q| q.iter().copied()).collect();

            let scoring = Instant::now();
            let scored = self.corpus.score(&flat, chunk.len())?;
            on_device += scoring.elapsed().as_secs_f64();

            let selecting = Instant::now();
            out.extend(scored.best_of_each(&self.corpus, chunk.len(), k, allowed));
            on_host += selecting.elapsed().as_secs_f64();
        }

        self.record_split(on_device, on_host, queries.len(), k, allowed.is_some());
        Ok(out)
    }

    fn search(
        &self,
        _store: &dyn VectorStore,
        query: &[f32],
        k: usize,
        allowed: Option<&dyn Fn(Ordinal) -> bool>,
    ) -> Result<Vec<Candidate>> {
        // `_store` is deliberately unused: this index owns a device-resident
        // copy of the corpus, so scoring never reads back through the store.
        // The parameter stays because it is the port's shape, and an index
        // that needed the store for reranking would use it.
        self.check_dim(query)?;
        if k == 0 || self.corpus.rows() == 0 {
            return Ok(Vec::new());
        }

        let scoring = Instant::now();
        let scored = self.corpus.score(query, 1)?;
        let on_device = scoring.elapsed().as_secs_f64();

        let selecting = Instant::now();
        let hits = scored.best(&self.corpus, 0, k, allowed);
        let on_host = selecting.elapsed().as_secs_f64();
        let elapsed = on_device + on_host;

        self.meter
            .histogram(metrics_names::SEARCH_DURATION, elapsed);
        self.meter
            .histogram(metrics_names::SEARCH_SCORE_DURATION, on_device);
        self.meter
            .histogram(metrics_names::SEARCH_SELECT_DURATION, on_host);
        self.meter
            .histogram(metrics_names::SEARCH_RESULTS, hits.len() as f64);

        logger::debug!("search complete").with_data(&serde_json::json!({
            fields::INDEX_KIND: self.kind(),
            fields::DEVICE: self.device,
            fields::K: k,
            fields::DIM: self.corpus.dim.get(),
            fields::FILTERED: allowed.is_some(),
            // Shape only, never values: a query vector can be inverted toward
            // its source text, and logs are read by people granted nothing
            // (invariant 28).
            fields::QUERY: redact::vector_shape(query),
            fields::RESULTS_RETURNED: hits.len(),
            fields::DURATION_SECONDS: elapsed,
            // The two halves, side by side. A single number cannot show that a
            // query spent its time on the wrong side of the split.
            fields::SCORE_SECONDS: on_device,
            fields::SELECT_SECONDS: on_host,
        }));
        Ok(hits)
    }
}

#[cfg(test)]
#[path = "search_test.rs"]
mod search_tests;

#[cfg(test)]
#[path = "batch_test.rs"]
mod batch_tests;
