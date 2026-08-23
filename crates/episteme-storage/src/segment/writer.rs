//! Sealing a buffer into an immutable segment.

use super::layout::{FieldLayout, field_dir};
use crate::error::Result;
use crate::format::quantize::{BinaryCodes, F16Row, Int8Row, PqCodebook, PqParams};
use crate::format::{Codec, FIELD_HEADER_BYTES, FieldHeader, HEADER_BYTES, SegmentHeader};
use episteme_core::{Fingerprint, VectorStore};
use episteme_telemetry::{fields, metrics_names};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Writes one sealed segment.
///
/// Everything is written into a temporary directory and moved into place only
/// once complete. A crash mid-seal therefore leaves a stray temp directory and
/// nothing else — never a partial segment that the manifest might later name.
pub struct SegmentWriter {
    tmp: PathBuf,
    final_path: PathBuf,
    schema_fingerprint: Fingerprint,
    rows: u64,
}

impl SegmentWriter {
    /// Begin a segment at `path`, which must not already exist.
    pub fn create(path: impl AsRef<Path>, schema_fingerprint: Fingerprint) -> Result<Self> {
        let final_path = path.as_ref().to_path_buf();
        let tmp = final_path.with_extension("building");
        if tmp.exists() {
            fs::remove_dir_all(&tmp)?;
        }
        fs::create_dir_all(&tmp)?;
        Ok(Self {
            tmp,
            final_path,
            schema_fingerprint,
            rows: 0,
        })
    }

    /// Write one named vector field at full precision, with no scan tier.
    pub fn write_field(
        &mut self,
        name: &str,
        store: &dyn VectorStore,
        model_fingerprint: Fingerprint,
    ) -> Result<()> {
        self.write_field_with_codec(name, store, model_fingerprint, Codec::None)
    }

    /// Write one named vector field, optionally with a compressed scan tier.
    ///
    /// Takes the store rather than raw bytes so the same call seals an unsealed
    /// buffer or re-writes an existing segment during compaction.
    ///
    /// When `codec` is not [`Codec::None`], a second file is written beside
    /// `raw.bin` holding the same rows compressed. That is the two-tier layout:
    /// scan wide and cheap over the codes, rescore the survivors at full
    /// precision. Both files carry every row at fixed stride, so an absent row
    /// still occupies its slot in each.
    pub fn write_field_with_codec(
        &mut self,
        name: &str,
        store: &dyn VectorStore,
        model_fingerprint: Fingerprint,
        codec: Codec,
    ) -> Result<()> {
        let span = tracing::info_span!(
            "episteme.segment.write_field",
            { fields::FIELD } = name,
            { fields::ROWS } = store.len(),
            { fields::DIM } = store.dim().get(),
        );
        let _guard = span.enter();

        let dir = field_dir(&self.tmp, name);
        fs::create_dir_all(&dir)?;

        let header = FieldHeader {
            dim: store.dim(),
            dtype: crate::format::DType::F32,
            codec,
            metric: store.metric(),
            model_fingerprint,
            rows: store.len() as u64,
        };
        let layout = FieldLayout::new(
            FIELD_HEADER_BYTES,
            header.raw_row_bytes(),
            store.len() as u64,
        );

        let mut file = fs::File::create(dir.join("raw.bin"))?;
        file.write_all(&header.encode())?;
        file.write_all(&vec![0u8; layout.padding(FIELD_HEADER_BYTES)])?;

        // A row absent for this field is written as zeros and excluded by the
        // presence bitmap. The bytes must still occupy their slot, or fixed
        // stride — and every offset computed from it — stops holding.
        let mut present = vec![0u8; store.len().div_ceil(8)];
        for row in 0..store.len() {
            let ordinal = episteme_core::Ordinal::from_row(row as u32);
            match store.get(ordinal) {
                Some(vector) => {
                    present[row / 8] |= 1 << (row % 8);
                    write_le_f32(&mut file, vector)?;
                }
                None => file.write_all(&vec![0u8; header.raw_row_bytes()])?,
            }
        }
        file.sync_all()?;

        fs::write(dir.join("present.roar"), &present)?;
        if codec != Codec::None {
            super::codes::write_codes(&dir, store, codec)?;
        }
        self.rows = self.rows.max(store.len() as u64);
        Ok(())
    }

    /// Write the segment header and move the segment into place.
    ///
    /// The directory rename is the moment the segment exists. Before it, a
    /// crash leaves only a temp directory; after it, the segment is complete.
    pub fn finish(self) -> Result<PathBuf> {
        let started = Instant::now();

        let header = SegmentHeader::new(self.schema_fingerprint, self.rows);
        let mut file = fs::File::create(self.tmp.join("header.bin"))?;
        file.write_all(&header.encode())?;
        file.sync_all()?;
        debug_assert_eq!(header.encode().len(), HEADER_BYTES);

        fs::rename(&self.tmp, &self.final_path)?;
        if let Some(parent) = self.final_path.parent() {
            let _ = fs::File::open(parent).and_then(|d| d.sync_all());
        }

        metrics::histogram!(metrics_names::SEGMENT_SEAL_DURATION)
            .record(started.elapsed().as_secs_f64());
        metrics::gauge!(metrics_names::ROWS_LIVE).set(self.rows as f64);
        tracing::info!({ fields::ROWS } = self.rows, "segment sealed");
        Ok(self.final_path)
    }
}

/// Write a float slice as explicit little-endian bytes.
///
/// Explicit rather than a pointer cast, for two reasons. The crate forbids
/// `unsafe`, and — more usefully — the segment format is little-endian by
/// definition, so writing it explicitly makes a big-endian host produce
/// *correct* files rather than silently byte-swapped ones. Sealing is not the
/// hot path; the read side is where zero-copy matters.
fn write_le_f32(file: &mut fs::File, vector: &[f32]) -> Result<()> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(vector));
    for value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    file.write_all(&bytes)?;
    Ok(())
}
