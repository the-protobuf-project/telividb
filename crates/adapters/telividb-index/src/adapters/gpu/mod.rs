//! Exhaustive search on the GPU, over a corpus held as a GGUF tensor.
//!
//! Exact, like [`FlatIndex`](crate::adapters::FlatIndex), and for the same
//! reason: it scores every row. What differs is where — one matmul on a
//! device rather than a loop on the host.
//!
//! **Why this is worth having next to HNSW.** Search is memory-bandwidth
//! bound, and on unified-memory Apple silicon the whole corpus is
//! GPU-addressable with no PCIe hop (AGENT_START §14.1), so exhaustive
//! scoring at that bandwidth is competitive with graph search at moderate
//! scale — while keeping an exactness guarantee HNSW cannot offer, costing no
//! build time, and never degrading under deletes.
//!
//! **Why HNSW is not ported here.** Graph traversal is sequential
//! pointer-chasing: each hop depends on the previous hop's result, and the
//! memory access pattern is unpredictable. That is the opposite of what a GPU
//! rewards, which is why FAISS ships no GPU HNSW either. HNSW stays a CPU
//! path; this complements it rather than replacing it.

mod budget;
mod detect;
mod device;
mod gguf;
mod search;

pub use budget::{
    BUDGET_ENV, BudgetSource, DEFAULT_FRACTION as DEFAULT_GPU_BUDGET_FRACTION, budget_source,
    device_allocated_bytes, limit_bytes as gpu_budget_bytes, resident_bytes as gpu_resident_bytes,
};
pub use device::{best_device, device_name};

use crate::domain::Candidate;
use crate::ports::{VectorIndex, VectorStore};
use candle_core::Device;
use std::time::Instant;
use telividb_core::{Ordinal, Result};
use telividb_telemetry::{Meter, fields, logger, metrics_names, redact};

/// A device-resident corpus, searched exhaustively.
pub struct GpuFlatIndex {
    corpus: gguf::Corpus,
    /// Registration in the shared residency registry, released on drop.
    ///
    /// Held rather than merely checked at construction: the ceiling has to
    /// account for every index alive at once, which is the case that actually
    /// killed the process — a rebuild holding the old corpus and the new one
    /// together.
    _reservation: telividb_telemetry::residency::Handle,
    /// Which device the corpus actually landed on, for telemetry.
    ///
    /// Recorded because a GPU index that silently fell back to CPU passes
    /// every correctness test while delivering none of the speed — the one
    /// failure this design can have that no assertion would catch.
    device: &'static str,
    /// Where search measurements go. Disabled until a composition root wires
    /// one in, so building an index needs no pipeline and no runtime.
    meter: Meter,
}

impl std::fmt::Debug for GpuFlatIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuFlatIndex")
            .field("device", &self.device)
            .field("rows", &self.corpus.present.len())
            .field("dim", &self.corpus.dim.get())
            .field("metric", &self.corpus.metric)
            .finish()
    }
}

impl GpuFlatIndex {
    /// Serialize `store` as GGUF bytes, ready to persist as
    /// `vectors/<field>/index.gguf`.
    ///
    /// Separate from opening so the index crate never touches a file — it
    /// hands bytes to storage, exactly as HNSW's `encode` does (invariant 6).
    pub fn encode(store: &dyn VectorStore) -> Result<Vec<u8>> {
        let mut buffer = std::io::Cursor::new(Vec::new());
        gguf::write_corpus(store, &mut buffer)?;
        Ok(buffer.into_inner())
    }

    /// Build directly from a store, choosing the best available device.
    ///
    /// Convenience over `encode` + `decode` for the common case; the bytes are
    /// still what gets persisted.
    pub fn build(store: &dyn VectorStore) -> Result<Self> {
        Self::decode(&Self::encode(store)?, &best_device())
    }

    /// Open a corpus written by [`GpuFlatIndex::encode`], onto `device`.
    pub fn decode(bytes: &[u8], device: &Device) -> Result<Self> {
        let started = Instant::now();
        let mut reader = std::io::Cursor::new(bytes);

        // Reserve *before* loading. An over-large upload does not fail
        // gracefully on Metal — it aborts the process — so this check is the
        // only point at which the failure is still recoverable.
        let reservation = budget::reserve("gpu-flat", bytes.len())?;

        let corpus = gguf::load_corpus(&mut reader, device)?;
        let name = device_name(device);

        logger::info!("gpu index loaded").with_data(&serde_json::json!({
            fields::INDEX_KIND: "gpu-flat",
            fields::DEVICE: name,
            fields::ROWS: corpus.present.len(),
            fields::DIM: corpus.dim.get(),
            fields::BYTES: reservation.bytes(),
            fields::DURATION_SECONDS: started.elapsed().as_secs_f64(),
        }));

        Ok(Self {
            corpus,
            _reservation: reservation,
            device: name,
            meter: Meter::disabled(),
        })
    }

    /// Record search measurements through `meter`.
    pub fn with_meter(mut self, meter: Meter) -> Self {
        self.meter = meter;
        self
    }

    /// Which device this corpus is resident on: `metal`, `cuda` or `cpu`.
    pub fn device(&self) -> &'static str {
        self.device
    }
}

impl VectorIndex for GpuFlatIndex {
    fn kind(&self) -> &'static str {
        "gpu-flat"
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
        let started = Instant::now();
        let hits = search::search(&self.corpus, query, k, allowed)?;
        let elapsed = started.elapsed().as_secs_f64();

        self.meter
            .histogram(metrics_names::SEARCH_DURATION, elapsed);
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
        }));
        Ok(hits)
    }
}
