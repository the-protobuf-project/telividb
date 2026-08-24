//! Rewriting segments without their tombstoned rows.
//!
//! The operation immutability makes safe: rather than editing a sealed segment,
//! compaction reads live rows from one or more inputs, writes a fresh segment,
//! and publishes it with a manifest swap. Readers holding the old manifest keep
//! a consistent view until they drop it, so nothing has to be locked and no
//! query is interrupted.
//!
//! **Ordinals do not survive.** A compacted segment renumbers its rows, which
//! is why an ordinal must never escape the process — anything that stored one
//! externally would now point at a different row. External identity is the
//! resource name; the ordinal is a position, and positions move.

use crate::buffer::MutableBuffer;
use crate::error::Result;
use episteme_core::{Ordinal, VectorStore};
use episteme_telemetry::{Meter, fields, logger, metrics_names};
use std::time::Instant;

/// What a compaction actually did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionResult {
    /// Rows read across every input.
    pub rows_read: u64,
    /// Rows written to the new segment.
    pub rows_written: u64,
    /// Rows dropped because they were tombstoned.
    ///
    /// Absent rows are *not* reclaimed: a row with no vector for this field
    /// still occupies its ordinal, because the fixed stride is what makes the
    /// read path a cast rather than a lookup. It is written and counted in
    /// [`rows_written`](Self::rows_written) like any other.
    pub rows_reclaimed: u64,
}

impl CompactionResult {
    /// Fraction of the input reclaimed. Near zero means the run was wasted.
    pub fn reclaimed_fraction(&self) -> f64 {
        if self.rows_read == 0 {
            return 0.0;
        }
        self.rows_reclaimed as f64 / self.rows_read as f64
    }
}

/// Merge one field of several sealed segments, dropping tombstoned rows.
///
/// `is_live` decides which rows survive, taking the input's index and the row's
/// ordinal within it. Passing the tombstone bitmap here rather than reading it
/// inside keeps compaction independent of how deletions are recorded.
///
/// Returns the merged rows as a buffer, ready to be sealed. Producing a buffer
/// rather than writing directly means the caller controls when the new segment
/// becomes visible — and the manifest swap stays the single publish point.
/// `meter` records the compaction duration; pass [`Meter::disabled`] when there
/// is no pipeline, which is what every test and embedded caller does.
///
/// Deliberately takes neither a schema fingerprint nor a codec. Both belong to
/// sealing rather than merging: the fingerprint is fixed by
/// [`SegmentWriter::create`](crate::SegmentWriter::create) and the codec by
/// `write_field_with_codec`, which the caller invokes on the buffer this
/// returns. Accepting them here and dropping them — which is what this
/// signature used to do — read as though a compacted field kept its scan tier
/// when nothing here could have written one.
pub fn compact_field(
    inputs: &[&dyn VectorStore],
    is_live: &dyn Fn(usize, Ordinal) -> bool,
    meter: &Meter,
) -> Result<(MutableBuffer, CompactionResult)> {
    let started = Instant::now();

    let Some(first) = inputs.first() else {
        let empty = MutableBuffer::new(
            episteme_core::Dim::new(1).expect("one is non-zero"),
            episteme_core::Metric::Dot,
        );
        return Ok((
            empty,
            CompactionResult {
                rows_read: 0,
                rows_written: 0,
                rows_reclaimed: 0,
            },
        ));
    };

    let total: usize = inputs.iter().map(|s| s.len()).sum();
    let mut out = MutableBuffer::with_capacity(first.dim(), first.metric(), total);
    let mut read = 0u64;
    let mut written = 0u64;

    for (i, store) in inputs.iter().enumerate() {
        for row in 0..store.len() {
            let ordinal = Ordinal::from_row(row as u32);
            read += 1;

            if !is_live(i, ordinal) {
                continue;
            }
            // An absent row carries no vector for this field. It survives as an
            // absent row rather than vanishing, so ordinals stay aligned across
            // the fields of the segment being written.
            match store.get(ordinal) {
                Some(vector) => {
                    out.push(vector)?;
                    written += 1;
                }
                None => {
                    out.push_absent();
                    written += 1;
                }
            }
        }
    }

    let result = CompactionResult {
        rows_read: read,
        rows_written: written,
        rows_reclaimed: read - written,
    };

    let elapsed = started.elapsed().as_secs_f64();
    meter.histogram(metrics_names::COMPACTION_DURATION, elapsed);
    // No `ROWS_TOMBSTONED` gauge here. Compacting one field of some segments
    // does not make the database's tombstone count zero, and setting it to zero
    // from here reported exactly that. The gauge belongs to whatever owns the
    // manifest and can see every segment.
    logger::info!("compaction complete").with_data(&serde_json::json!({
        fields::ROWS: result.rows_read,
        fields::ROWS_WRITTEN: written,
        fields::ROWS_RECLAIMED: result.rows_reclaimed,
        fields::DURATION_SECONDS: elapsed,
    }));
    Ok((out, result))
}

#[cfg(test)]
#[path = "compact_test.rs"]
mod tests;
