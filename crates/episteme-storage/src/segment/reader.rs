//! Reading a sealed segment back.

use super::layout::{FieldLayout, field_dir};
use crate::error::{Error, Result};
use crate::format::{FIELD_HEADER_BYTES, FieldHeader, HEADER_BYTES, SegmentHeader};
use crate::ports::{BlockReader, FileBlockReader};
use episteme_core::{Dim, Fingerprint, Metric, Ordinal, VectorStore};
use std::path::Path;

/// One named vector field of a sealed segment, loaded and searchable.
///
/// Implements [`VectorStore`], so an index cannot tell a sealed field from the
/// unsealed buffer — which is exactly the point of the port.
///
/// `Debug` prints metadata only. Dumping the vectors would put them in logs,
/// which is the leak the telemetry rules exist to prevent.
pub struct SegmentReader {
    header: FieldHeader,
    layout: FieldLayout,
    rows: Vec<f32>,
    present: Vec<u8>,
}

impl std::fmt::Debug for SegmentReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SegmentReader")
            .field("dim", &self.header.dim.get())
            .field("metric", &self.header.metric)
            .field("rows", &self.header.rows)
            .field("model", &self.header.model_fingerprint)
            .finish_non_exhaustive()
    }
}

impl SegmentReader {
    /// Open one field of the segment at `path`.
    ///
    /// Validates the segment header's schema fingerprint against `schema`
    /// before reading anything: a segment written under an incompatible schema
    /// must be refused, not parsed and quietly misinterpreted.
    pub fn open_field(path: impl AsRef<Path>, field: &str, schema: Fingerprint) -> Result<Self> {
        let path = path.as_ref();

        let segment_header = read_segment_header(path)?;
        segment_header.check_schema(schema)?;

        let dir = field_dir(path, field);
        let reader = FileBlockReader::open(dir.join("raw.bin"))?;

        let mut header_bytes = [0u8; FIELD_HEADER_BYTES];
        reader.read_exact_at(0, &mut header_bytes)?;
        let header = FieldHeader::decode(&header_bytes)?;

        let layout = FieldLayout::new(FIELD_HEADER_BYTES, header.raw_row_bytes(), header.rows);
        let present = std::fs::read(dir.join("present.roar")).unwrap_or_default();

        let mut rows = vec![0f32; header.rows as usize * header.dim.get()];
        let mut buf = vec![0u8; header.raw_row_bytes()];
        for row in 0..header.rows {
            reader.read_exact_at(layout.row_offset(row), &mut buf)?;
            let start = row as usize * header.dim.get();
            for (i, chunk) in buf.chunks_exact(4).enumerate() {
                rows[start + i] = f32::from_le_bytes(chunk.try_into().expect("4 bytes"));
            }
        }

        Ok(Self {
            header,
            layout,
            rows,
            present,
        })
    }

    pub fn header(&self) -> &FieldHeader {
        &self.header
    }

    pub fn layout(&self) -> FieldLayout {
        self.layout
    }

    /// Refuse vectors produced by a different model than the one configured.
    pub fn check_model(&self, current: Fingerprint) -> Result<()> {
        self.header.check_model(current)
    }

    fn is_present(&self, row: usize) -> bool {
        // An absent presence bitmap means "everything present" — that is the
        // shape of a field written before presence was tracked, and treating a
        // missing file as "nothing present" would silently empty the field.
        if self.present.is_empty() {
            return true;
        }
        self.present
            .get(row / 8)
            .is_some_and(|byte| byte & (1 << (row % 8)) != 0)
    }
}

/// Read and validate the segment-level header.
fn read_segment_header(path: &Path) -> Result<SegmentHeader> {
    let reader = FileBlockReader::open(path.join("header.bin"))?;
    if reader.len() < HEADER_BYTES as u64 {
        return Err(Error::Truncated {
            what: "segment header",
            needed: HEADER_BYTES,
            found: reader.len() as usize,
        });
    }
    let mut bytes = [0u8; HEADER_BYTES];
    reader.read_exact_at(0, &mut bytes)?;
    SegmentHeader::decode(&bytes)
}

impl VectorStore for SegmentReader {
    fn dim(&self) -> Dim {
        self.header.dim
    }

    fn metric(&self) -> Metric {
        self.header.metric
    }

    fn len(&self) -> usize {
        self.header.rows as usize
    }

    fn get(&self, ordinal: Ordinal) -> Option<&[f32]> {
        let row = ordinal.row() as usize;
        if row >= self.len() || !self.is_present(row) {
            return None;
        }
        let start = row * self.header.dim.get();
        self.rows.get(start..start + self.header.dim.get())
    }
}
