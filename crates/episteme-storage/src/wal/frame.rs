//! Record framing: `len(u32) crc32(u32) payload`.

use crate::error::{Error, Result};

pub const FRAME_HEADER_BYTES: usize = 8;

/// Serialize one record's frame header.
pub(crate) fn encode_header(payload: &[u8]) -> [u8; FRAME_HEADER_BYTES] {
    let mut out = [0u8; FRAME_HEADER_BYTES];
    out[0..4].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    out[4..8].copy_from_slice(&crc32fast::hash(payload).to_le_bytes());
    out
}

/// Parsed frame header.
pub(crate) struct FrameHeader {
    pub len: usize,
    pub crc: u32,
}

pub(crate) fn decode_header(bytes: &[u8; FRAME_HEADER_BYTES]) -> FrameHeader {
    FrameHeader {
        len: u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")) as usize,
        crc: u32::from_le_bytes(bytes[4..8].try_into().expect("4 bytes")),
    }
}

/// Verify a payload against the checksum recorded for it.
pub(crate) fn verify(payload: &[u8], expected: u32) -> Result<()> {
    let computed = crc32fast::hash(payload);
    if computed != expected {
        return Err(Error::Corrupt {
            what: "wal record",
            expected,
            computed,
        });
    }
    Ok(())
}
