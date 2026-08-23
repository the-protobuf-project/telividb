//! Appending to the log.

use super::frame;
use crate::error::Result;
use episteme_telemetry::{fields, metrics_names};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

/// Append-only writer over one WAL file.
///
/// Buffered, because the cost that matters is `fsync`, not `write`. Call
/// [`WalWriter::commit`] to make everything appended so far durable — batching
/// many records into one commit is the whole point.
pub struct WalWriter {
    file: BufWriter<File>,
    pending: usize,
    pending_bytes: u64,
}

impl WalWriter {
    /// Open for append, creating the file if absent.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            file: BufWriter::new(file),
            pending: 0,
            pending_bytes: 0,
        })
    }

    /// Append one record. Not durable until [`WalWriter::commit`].
    pub fn append(&mut self, payload: &[u8]) -> Result<()> {
        let header = frame::encode_header(payload);
        self.file.write_all(&header)?;
        self.file.write_all(payload)?;
        self.pending += 1;
        self.pending_bytes += (header.len() + payload.len()) as u64;
        Ok(())
    }

    /// Records appended since the last commit.
    pub fn pending(&self) -> usize {
        self.pending
    }

    /// Flush and `fsync`. Everything appended so far survives a crash.
    ///
    /// Instrumented because `fsync` latency is the write path's whole cost, and
    /// records-per-commit is what says whether group commit is actually
    /// batching or degenerating into one fsync per row.
    pub fn commit(&mut self) -> Result<()> {
        let span = tracing::debug_span!(
            "episteme.wal.commit",
            { fields::RECORDS } = self.pending,
            { fields::BYTES } = self.pending_bytes,
        );
        let _guard = span.enter();

        if self.pending == 0 {
            return Ok(());
        }
        let started = Instant::now();

        self.file.flush()?;
        self.file.get_ref().sync_data()?;

        metrics::histogram!(metrics_names::WAL_COMMIT_DURATION)
            .record(started.elapsed().as_secs_f64());
        metrics::histogram!(metrics_names::WAL_COMMIT_RECORDS).record(self.pending as f64);
        metrics::counter!(metrics_names::WAL_BYTES).increment(self.pending_bytes);

        self.pending = 0;
        self.pending_bytes = 0;
        Ok(())
    }
}

impl Drop for WalWriter {
    fn drop(&mut self) {
        // Best-effort only. A caller that needs durability calls commit and
        // handles its error; silently succeeding here would hide data loss.
        let _ = self.commit();
    }
}
