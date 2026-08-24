//! Turning the live buffer into an immutable segment.
//!
//! Split from `mod.rs` because sealing is the one operation that changes a
//! field's *shape* rather than its contents: rows move from memory to a file,
//! the manifest gains a segment, and the log starts over. Everything else in
//! that file appends or reads.
//!
//! Ordering here is load-bearing. The segment is written and synced **before**
//! the manifest names it, so a crash between the two leaves an unreferenced
//! directory — wasted space — rather than a manifest pointing at something
//! that does not exist. The log is truncated **after** both, because until the
//! segment is visible the log is still the only durable copy.

use super::VectorField;
use super::segment_dir;
use crate::error::Result;
use crate::segment::{SegmentReader, SegmentWriter};
use crate::wal::WalWriter;
use telividb_core::VectorStore;

impl VectorField {
    /// Seal the buffer into a segment if it has grown past `threshold`.
    ///
    /// Returns whether it sealed. The manifest swap is what makes the segment
    /// visible, and it happens only after every byte is on disk.
    pub fn seal_if_needed(&mut self, threshold: usize) -> Result<bool> {
        if !self.buffer.should_seal(threshold) || self.buffer.rows() == 0 {
            return Ok(false);
        }
        self.seal()?;
        Ok(true)
    }

    /// Seal the buffer into a segment regardless of size.
    pub fn seal(&mut self) -> Result<()> {
        let id = self.manifest.segments.len() as u64 + 1;
        let path = segment_dir(&self.dir, id);

        let mut writer = SegmentWriter::create(&path, self.schema)?;
        writer.write_field(&self.field, &self.buffer, self.model)?;
        writer.finish()?;

        // The segment exists on disk before the manifest names it, so a crash
        // between the two leaves an unreferenced directory rather than a
        // manifest pointing at nothing.
        self.manifest = self.manifest.clone().with_segment(id);
        self.manifest
            .write_atomic(self.dir.join("MANIFEST"), &Default::default())?;

        let reader = SegmentReader::open_field(&path, &self.field, self.schema)?;
        self.sealed_rows += reader.len();
        self.sealed.push(reader);
        self.buffer.clear();

        // The log's records are all in the segment now, so the file can start
        // over — otherwise replay would re-add rows the segment already holds.
        let wal_path = self.dir.join("000001.wal");
        std::fs::remove_file(&wal_path).ok();
        self.wal = WalWriter::open(&wal_path)?;
        Ok(())
    }
}
