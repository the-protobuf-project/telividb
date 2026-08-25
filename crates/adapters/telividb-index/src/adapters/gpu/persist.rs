//! Writing an index to bytes and reading it back.
//!
//! Split from `mod.rs` because it is the *persistence* half: `mod.rs` builds
//! and searches an index in memory, and these two turn one into a versioned
//! byte stream and back. Keeping them apart is what makes the in-memory build
//! path obviously free of any file format — see `direct.rs` for why that
//! mattered.

use super::{GpuFlatIndex, budget, device_name, gguf, load};
use candle_core::Device;
use std::time::Instant;
use telividb_core::{Result, VectorStore};
use telividb_telemetry::{Meter, fields, logger};

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

    /// Open a corpus written by [`GpuFlatIndex::encode`], onto `device`.
    pub fn decode(bytes: &[u8], device: &Device) -> Result<Self> {
        let started = Instant::now();
        let mut reader = std::io::Cursor::new(bytes);

        // Reserve *before* loading. An over-large upload does not fail
        // gracefully on Metal — it aborts the process — so this check is the
        // only point at which the failure is still recoverable.
        let reservation = budget::reserve("gpu-flat", bytes.len())?;

        let corpus = load::load_corpus(&mut reader, device)?;
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
}
