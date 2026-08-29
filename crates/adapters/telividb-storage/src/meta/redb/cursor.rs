//! A bounds-checked reader over a stored record.
//!
//! Split from `collection_record.rs` so that file is about the layout and this
//! one about reading it safely.
//!
//! Every read is checked because these bytes come from disk, where a truncated
//! write is a real outcome — an unchecked slice would panic inside a request
//! handler rather than report a corrupt record.

use telividb_core::{Error, Result};

/// A bounds-checked reader over the record.
///
/// Every read is checked because these bytes come from disk, where a truncated
/// write is a real outcome — an unchecked slice would panic inside a request
/// handler rather than report a corrupt record.
pub(super) struct Cursor<'a> {
    /// The record being read.
    pub(super) bytes: &'a [u8],
    /// How far into it the reader has advanced.
    pub(super) offset: usize,
}

impl Cursor<'_> {
    /// Advance by `n` bytes, or report the record as truncated.
    pub(super) fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self.offset.checked_add(n).ok_or_else(|| truncated(n))?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| truncated(n))?;
        self.offset = end;
        Ok(slice)
    }

    /// The next byte — a version marker or an enum discriminant.
    pub(super) fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    /// A little-endian `u32`.
    pub(super) fn u32(&mut self) -> Result<u32> {
        let raw = self.take(4)?;
        Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    /// A 32-byte digest.
    pub(super) fn array32(&mut self) -> Result<[u8; 32]> {
        let raw = self.take(32)?;
        let mut out = [0u8; 32];
        out.copy_from_slice(raw);
        Ok(out)
    }

    /// A length-prefixed byte run.
    pub(super) fn bytes(&mut self) -> Result<&[u8]> {
        let len = self.u32()? as usize;
        self.take(len)
    }

    /// A length-prefixed UTF-8 string.
    pub(super) fn string(&mut self) -> Result<String> {
        let raw = self.bytes()?;
        String::from_utf8(raw.to_vec()).map_err(|e| Error::PointStore {
            reason: format!("collection record holds invalid utf-8: {e}"),
        })
    }
}

/// The "ran off the end" error, shared by every read.
fn truncated(needed: usize) -> Error {
    Error::PointStore {
        reason: format!("collection record is truncated: needed {needed} more bytes"),
    }
}
