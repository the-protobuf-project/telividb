//! One named vector field's durable write path.
//!
//! Ties together the four pieces that already existed separately — WAL,
//! mutable buffer, sealed segments, manifest — into the lifecycle
//! ARCHITECTURE §4.1 describes:
//!
//! ```text
//! append → WAL (durable) → buffer (searchable) → seal → segment → manifest
//! ```
//!
//! **Why the WAL comes first.** A vector in the buffer and nowhere else is
//! lost when the process dies, and the buffer is where every write lives until
//! a seal threshold is crossed — which for a small collection may be never.
//! Appending to the log before the buffer is what makes a write survive a
//! crash the instant it is acknowledged.
//!
//! **Why the buffer is searchable rather than write-only.** A write must be
//! findable immediately; waiting for a seal would make "I just wrote it and
//! cannot find it" the normal experience for interactive ingest. So
//! [`VectorField::stores`] hands back the buffer *and* every sealed segment,
//! and the caller searches all of them and merges.
//!
//! **What recovery does.** [`VectorField::open`] replays the log into the
//! buffer. A torn tail — the process died mid-append — is truncated rather
//! than guessed at: the partial record never happened, and every intact record
//! before it is restored.

mod meta;
mod record;
mod recover;
mod seal;

pub use meta::FieldMeta;

use crate::buffer::MutableBuffer;
use crate::error::Result;
use crate::manifest::Manifest;
use crate::segment::SegmentReader;
use crate::wal::WalWriter;
use std::path::{Path, PathBuf};
use telividb_core::{Dim, Fingerprint, Metric, Ordinal, VectorStore};

/// Seal once the buffer holds this much. Deliberately modest: a larger buffer
/// means more to replay after a crash and more memory held before anything is
/// mmap-able.
pub const DEFAULT_SEAL_BYTES: usize = 64 * 1024 * 1024;

/// One vector field of one collection, durable across restarts.
pub struct VectorField {
    dir: PathBuf,
    field: String,
    wal: WalWriter,
    buffer: MutableBuffer,
    manifest: Manifest,
    sealed: Vec<SegmentReader>,
    schema: Fingerprint,
    model: Fingerprint,
    /// Rows in sealed segments — the offset the buffer's ordinals start at.
    ///
    /// Segment ordinals are segment-local (invariant 9), so a caller that
    /// searches several stores needs this to turn a hit into a row number that
    /// means something across the whole field.
    sealed_rows: usize,
}

impl VectorField {
    /// Open the field under `dir`, recovering anything the log still holds.
    pub fn open(
        dir: impl AsRef<Path>,
        field: &str,
        dim: Dim,
        metric: Metric,
        schema: Fingerprint,
        model: Fingerprint,
    ) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;

        // The field's own dimension and metric, written once and read back
        // thereafter. Without this a caller has to *supply* the width to open
        // an existing field — and supplying the wrong one silently rejects
        // every record replayed from the log, which looks like an empty field
        // rather than a mismatch.
        let (dim, metric) = match meta::read(&dir)? {
            Some(found) => {
                found.check(dim, metric)?;
                (found.dim, found.metric)
            }
            None => {
                meta::write(&dir, FieldMeta { dim, metric })?;
                (dim, metric)
            }
        };

        let manifest = Manifest::read(dir.join("MANIFEST")).unwrap_or_default();
        let mut sealed = Vec::new();
        let mut sealed_rows = 0usize;
        for &id in &manifest.segments {
            let reader = SegmentReader::open_field(segment_dir(&dir, id), field, schema)?;
            reader.check_model(model)?;
            sealed_rows += reader.len();
            sealed.push(reader);
        }

        let (buffer, wal) = recover::replay(&dir, field, dim, metric)?;

        Ok(Self {
            wal,
            dir,
            field: field.to_owned(),
            buffer,
            manifest,
            sealed,
            schema,
            model,
            sealed_rows,
        })
    }

    /// Append one vector, returning the row it occupies across the whole field.
    ///
    /// The WAL is written first and the buffer second, so a crash between them
    /// loses nothing: replay puts the record back.
    pub fn append(&mut self, vector: &[f32]) -> Result<usize> {
        self.wal.append(&record::encode(vector))?;
        let ordinal = self.buffer.push(vector)?;
        Ok(self.sealed_rows + ordinal.row() as usize)
    }

    /// Make every appended vector durable.
    ///
    /// Separate from [`VectorField::append`] so a batch pays one fsync rather
    /// than one per row — the group-commit shape ARCHITECTURE §4.1 describes.
    pub fn commit(&mut self) -> Result<()> {
        self.wal.commit()
    }

    /// Every store holding rows of this field: sealed segments first, then the
    /// unsealed buffer.
    ///
    /// Order matters and is part of the contract — it is what makes
    /// [`VectorField::row_of`] able to turn a `(store, ordinal)` pair into a
    /// row number spanning the whole field.
    pub fn stores(&self) -> Vec<&dyn VectorStore> {
        let mut stores: Vec<&dyn VectorStore> = Vec::with_capacity(self.sealed.len() + 1);
        for reader in &self.sealed {
            stores.push(reader);
        }
        stores.push(&self.buffer);
        stores
    }

    /// Turn a hit in store `index` at `ordinal` into a field-wide row number.
    ///
    /// Segment ordinals are segment-local, so this is the only correct way to
    /// compare a hit from one store against a hit from another.
    pub fn row_of(&self, index: usize, ordinal: Ordinal) -> usize {
        let base: usize = self.sealed.iter().take(index).map(|r| r.len()).sum();
        base + ordinal.row() as usize
    }

    /// The width every vector in this field has.
    ///
    /// Read from the field itself rather than inferred from whatever vector is
    /// at hand: a query of the wrong width must be refused, not used to
    /// reinterpret a persisted segment.
    pub fn dim(&self) -> Dim {
        self.buffer.dim()
    }

    /// Rows across sealed segments and the buffer together.
    pub fn rows(&self) -> usize {
        self.sealed_rows + self.buffer.rows()
    }

    /// Bytes the unsealed buffer currently holds.
    pub fn buffered_bytes(&self) -> usize {
        self.buffer.bytes()
    }
}

/// Where segment `id` lives under a field directory.
fn segment_dir(dir: &Path, id: u64) -> PathBuf {
    dir.join(format!("seg_{id:05}"))
}

#[cfg(test)]
#[path = "field_test.rs"]
mod tests;
