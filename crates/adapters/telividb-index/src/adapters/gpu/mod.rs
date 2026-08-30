//! Exhaustive search on a device, over a corpus held as one resident matrix.
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
//!
//! **The corpus is rebuilt, never persisted.** It is exactly derivable from
//! the store, and rebuilding a million rows measures at 0.14 s against the
//! 512 MB a serialized copy would occupy — so persisting it would trade real
//! disk for no recovery time. Nothing here writes a file; rule 4 has nothing
//! to version because there is no on-disk structure.

mod budget;
mod corpus;
mod error;
mod host;
mod metric;
mod scan;
mod scored;
mod search;
mod select;

#[cfg(test)]
mod test_support;

pub use budget::{
    BUDGET_ENV, BudgetSource, DEFAULT_FRACTION as DEFAULT_GPU_BUDGET_FRACTION, budget_source,
    device_allocated_bytes, device_name, limit_bytes as gpu_budget_bytes,
    resident_bytes as gpu_resident_bytes,
};

use crate::ports::VectorStore;
use corpus::DeviceCorpus;
use error::OnDevice;
use std::time::Instant;
use telividb_compute::{Backend, DeviceKind};
use telividb_core::Result;
use telividb_telemetry::{Meter, fields, logger, metrics_names};

/// A device-resident corpus, searched exhaustively.
pub struct GpuFlatIndex {
    corpus: DeviceCorpus,
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
            .field("rows", &self.corpus.rows())
            .field("dim", &self.corpus.dim.get())
            .field("metric", &self.corpus.metric)
            .finish()
    }
}

impl GpuFlatIndex {
    /// Build from a store, on the fastest device this build can reach.
    pub fn build(store: &dyn VectorStore) -> Result<Self> {
        Self::open(store, Backend::best().on_device()?)
    }

    /// Build from a store on a chosen device kind.
    ///
    /// Fails if that kind was not compiled in, rather than falling back — a
    /// caller who names a device is testing or benchmarking that device, and a
    /// silent substitution would make the measurement meaningless.
    pub fn build_on(store: &dyn VectorStore, kind: DeviceKind) -> Result<Self> {
        Self::open(store, Backend::of(kind).on_device()?)
    }

    /// Upload `store` to `backend` and account for it.
    fn open(store: &dyn VectorStore, backend: Backend) -> Result<Self> {
        let started = Instant::now();
        let device = backend.device().kind().as_str();

        // Reserved *before* the upload: an over-large one aborts the process on
        // Metal rather than failing, so this is the last recoverable point.
        let reservation = budget::reserve("gpu-flat", DeviceCorpus::device_bytes(store))?;
        let corpus = DeviceCorpus::from_store(store, backend)?;

        logger::info!("gpu index built").with_data(&serde_json::json!({
            fields::INDEX_KIND: "gpu-flat",
            fields::DEVICE: device,
            fields::ROWS: corpus.rows(),
            fields::DIM: corpus.dim.get(),
            fields::BYTES: reservation.bytes(),
            fields::DURATION_SECONDS: started.elapsed().as_secs_f64(),
        }));

        Ok(Self {
            corpus,
            device,
            _reservation: reservation,
            meter: Meter::disabled(),
        })
    }

    /// Record search measurements through `meter`.
    pub fn with_meter(mut self, meter: Meter) -> Self {
        self.meter = meter;
        self
    }

    /// Which device this corpus is resident on: `metal`, `cuda`, `cpu`, …
    pub fn device(&self) -> &'static str {
        self.device
    }

    /// How the host half of a batched search is executed: `parallel` or
    /// `serial`.
    ///
    /// Reported for the same reason the device is (rule 46): selection is the
    /// larger half of a batched query, so a build that quietly lost the
    /// `parallel` feature answers every query correctly at roughly half the
    /// throughput — and nothing else would surface that.
    pub fn selection() -> &'static str {
        match cfg!(feature = "parallel") {
            true => "parallel",
            false => "serial",
        }
    }

    /// Report where a batch spent its time, device against host.
    ///
    /// Recorded for the batch rather than per query because that is the unit
    /// the device call actually covers — attributing one matmul across 32
    /// queries would invent a per-query number the hardware never produced.
    fn record_split(&self, on_device: f64, on_host: f64, queries: usize, k: usize, filtered: bool) {
        self.meter
            .histogram(metrics_names::SEARCH_DURATION, on_device + on_host);
        self.meter
            .histogram(metrics_names::SEARCH_SCORE_DURATION, on_device);
        self.meter
            .histogram(metrics_names::SEARCH_SELECT_DURATION, on_host);

        logger::debug!("batch search complete").with_data(&serde_json::json!({
            fields::INDEX_KIND: "gpu-flat",
            fields::DEVICE: self.device,
            fields::K: k,
            fields::DIM: self.corpus.dim.get(),
            fields::FILTERED: filtered,
            fields::QUERIES: queries,
            fields::DURATION_SECONDS: on_device + on_host,
            fields::SCORE_SECONDS: on_device,
            fields::SELECT_SECONDS: on_host,
        }));
    }

    /// Reject a query whose width is not the corpus's.
    fn check_dim(&self, query: &[f32]) -> Result<()> {
        let dim = self.corpus.dim.get();
        match query.len() {
            len if len == dim => Ok(()),
            actual => Err(telividb_core::Error::DimMismatch {
                expected: dim,
                actual,
            }),
        }
    }
}
