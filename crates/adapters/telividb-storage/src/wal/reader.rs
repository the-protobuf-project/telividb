//! Replaying the log.

use super::frame::{self, FRAME_HEADER_BYTES};
use crate::error::Result;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use telividb_telemetry::{Meter, fields, logger, metrics_names};

/// How reading the log ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalTail {
    /// Every record was intact and the file ended on a record boundary.
    Clean,
    /// The final record was incomplete — the process died mid-write. Records
    /// before it are still good; this one never happened.
    Torn {
        /// Byte offset of the last intact record boundary.
        at_offset: u64,
    },
}

/// Sequential reader over one WAL file.
pub struct WalReader {
    inner: BufReader<File>,
    offset: u64,
    /// Where torn-tail recoveries are counted. Disabled until a composition
    /// root wires one in, so replaying a log never needs a pipeline.
    meter: Meter,
}

impl WalReader {
    /// Open a log for sequential replay.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            inner: BufReader::new(File::open(path)?),
            offset: 0,
            meter: Meter::disabled(),
        })
    }

    /// Count torn-tail recoveries through `meter`.
    ///
    /// Separate from [`WalReader::open`] so that opening a log stays a
    /// filesystem operation, with no pipeline and no runtime involved.
    pub fn with_meter(mut self, meter: Meter) -> Self {
        self.meter = meter;
        self
    }

    /// Replay every intact record, reporting how the file ended.
    ///
    /// A torn tail is not an error. It is the expected shape of a log written
    /// by a process that was killed, and recovery must proceed from the last
    /// good record rather than refusing to start.
    pub fn replay(mut self, mut on_record: impl FnMut(&[u8])) -> Result<WalTail> {
        let mut records = 0u64;

        loop {
            let mut header = [0u8; FRAME_HEADER_BYTES];
            match read_full(&mut self.inner, &mut header)? {
                Fill::Eof => {
                    logger::info!("wal replay clean")
                        .with_data(&serde_json::json!({ fields::RECORDS: records }));
                    return Ok(WalTail::Clean);
                }
                Fill::Partial => return Ok(self.torn(records)),
                Fill::Complete => {}
            }

            let parsed = frame::decode_header(&header);
            let mut payload = vec![0u8; parsed.len];
            if read_full(&mut self.inner, &mut payload)? != Fill::Complete {
                return Ok(self.torn(records));
            }

            // A complete-but-corrupt record is different from a torn one: the
            // bytes are all there and they are wrong, which means the media
            // lied rather than the process dying. Surface it.
            frame::verify(&payload, parsed.crc)?;

            on_record(&payload);
            records += 1;
            self.offset += (FRAME_HEADER_BYTES + parsed.len) as u64;
        }
    }

    /// Report a torn tail once, in one place.
    ///
    /// Counted rather than merely logged: after a hard kill this is expected,
    /// but a rising rate in steady state means the host is losing writes and
    /// that deserves an alert rather than a line in a log nobody reads.
    fn torn(&self, records: u64) -> WalTail {
        self.meter.counter(metrics_names::WAL_TORN_RECOVERIES, 1.0);
        logger::warn!("wal tail was torn; recovering to last intact record").with_data(
            &serde_json::json!({
                fields::RECORDS: records,
                fields::OFFSET: self.offset,
            }),
        );
        WalTail::Torn {
            at_offset: self.offset,
        }
    }
}

#[derive(PartialEq, Eq)]
enum Fill {
    Complete,
    Partial,
    Eof,
}

fn read_full(reader: &mut impl Read, buf: &mut [u8]) -> Result<Fill> {
    let mut filled = 0;
    while filled < buf.len() {
        match reader.read(&mut buf[filled..])? {
            0 => {
                return Ok(if filled == 0 {
                    Fill::Eof
                } else {
                    Fill::Partial
                });
            }
            n => filled += n,
        }
    }
    Ok(Fill::Complete)
}
