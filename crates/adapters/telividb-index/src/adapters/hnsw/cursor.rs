//! Reading a graph file without trusting it.
//!
//! A graph file is untrusted input the moment an archive arrives from
//! elsewhere. Every read is bounds-checked so a length field lying about its
//! size produces an error rather than a panic or an out-of-bounds read.

use telividb_core::{Error, Result};

/// Bounds-checked sequential reader.
///
/// Every read is checked because a graph file is untrusted input the moment an
/// archive arrives from elsewhere — a length field lying about its size must
/// produce an error, never a panic or an out-of-bounds read.
pub(super) struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    /// Wrap a byte slice for bounds-checked sequential reads.
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    /// Bytes not yet consumed. Every length read from the file is checked
    /// against this before it is used to allocate.
    pub(super) fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.at)
    }

    /// Consume and return the next `n` bytes, or error if they do not exist.
    pub(super) fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).ok_or(Error::MalformedIndex {
            reason: "length overflow",
        })?;
        let slice = self.bytes.get(self.at..end).ok_or(Error::MalformedIndex {
            reason: "truncated",
        })?;
        self.at = end;
        Ok(slice)
    }

    /// Read a little-endian u16, advancing by 2 bytes.
    pub(super) fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("2 bytes"),
        ))
    }

    /// Read a little-endian u32, advancing by 4 bytes.
    pub(super) fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }

    /// Read a little-endian u64, advancing by 8 bytes.
    pub(super) fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 bytes"),
        ))
    }
}
