//! Rebuilding a field's live buffer from its write-ahead log.
//!
//! Split from `mod.rs` because recovery is a distinct phase: it runs once, at
//! open, before the field accepts anything new — where the rest of that file is
//! the steady-state append and read surface.
//!
//! Two properties are load-bearing here.
//!
//! **A torn tail is truncated, not appended past.** `WalWriter::open` appends,
//! so writing after a partial record would put good records behind a broken
//! one — and every later replay would stop at that same break, permanently
//! losing everything written after it.
//!
//! **A record that cannot be restored is counted, not ignored.** Silently
//! skipping one makes a lossy recovery look complete, which is the failure
//! nobody notices until the data is missing.

use crate::buffer::MutableBuffer;
use crate::error::Result;
use crate::wal::{WalReader, WalTail, WalWriter};
use std::path::Path;
use telividb_core::{Dim, Metric};
use telividb_telemetry::{fields, logger};

/// Replay `dir`'s log into a fresh buffer, returning it and a writer
/// positioned to append.
pub(super) fn replay(
    dir: &Path,
    field: &str,
    dim: Dim,
    metric: Metric,
) -> Result<(MutableBuffer, WalWriter)> {
    let mut buffer = MutableBuffer::new(dim, metric);
    let wal_path = dir.join("000001.wal");

    if wal_path.exists() {
        let mut recovered = 0usize;
        let mut rejected = 0usize;
        let tail = WalReader::open(&wal_path)?.replay(|bytes| {
            match super::record::decode(bytes, dim.get()) {
                Some(vector) if buffer.push(&vector).is_ok() => recovered += 1,
                _ => rejected += 1,
            }
        })?;

        if let WalTail::Torn { at_offset } = tail {
            std::fs::OpenOptions::new()
                .write(true)
                .open(&wal_path)?
                .set_len(at_offset)?;
        }

        logger::info!("wal replayed").with_data(&serde_json::json!({
            fields::FIELD: field,
            fields::RECORDS: recovered,
            fields::REJECTED: rejected,
            fields::INCOMPLETE_REASON: match tail {
                WalTail::Clean => "none",
                WalTail::Torn { .. } => "torn_tail",
            },
        }));
    }

    Ok((buffer, WalWriter::open(&wal_path)?))
}
