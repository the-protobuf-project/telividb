//! A bounds-checked cursor over a GGUF header prefix.

use crate::{Error, Result};

/// Reads little-endian scalars and length-prefixed strings, refusing to run
/// past the end of the buffer.
///
/// The buffer is normally a *prefix* of the file — a range request rather than
/// a download — so running out of bytes is expected rather than exceptional.
/// [`Cursor::ended`] distinguishes that from malformed input, which lets the
/// parser keep what it read and stop.
pub(super) struct Cursor<'a> {
    /// The bytes available.
    bytes: &'a [u8],
    /// How far in the cursor has read.
    at: usize,
    /// Whether a read ran past the end.
    ended: bool,
}

impl<'a> Cursor<'a> {
    /// Start at the beginning of `bytes`.
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            at: 0,
            ended: false,
        }
    }

    /// Whether a read has run past the end of the buffer.
    pub(super) fn ended(&self) -> bool {
        self.ended
    }

    /// Take `n` bytes, or record that the buffer ended.
    pub(super) fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(n)?;
        match self.bytes.get(self.at..end) {
            Some(slice) => {
                self.at = end;
                Some(slice)
            }
            None => {
                self.ended = true;
                None
            }
        }
    }

    /// Read a little-endian `u32`.
    pub(super) fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    /// Read a little-endian `u64`.
    pub(super) fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    /// Read a length-prefixed UTF-8 string.
    ///
    /// Lossy rather than strict: a metadata key with a stray byte should not
    /// stop the parse, because the fields that matter here are ASCII and the
    /// alternative is refusing a file over a token nobody reads.
    pub(super) fn string(&mut self) -> Option<String> {
        let len = usize::try_from(self.u64()?).ok()?;
        Some(String::from_utf8_lossy(self.take(len)?).into_owned())
    }

    /// Confirm the GGUF magic and step over the file header.
    ///
    /// Returns the number of metadata pairs that follow.
    pub(super) fn header(&mut self) -> Result<u64> {
        let magic = self
            .take(4)
            .ok_or_else(|| Error::Gguf("the file is shorter than a GGUF header".to_owned()))?;
        if magic != b"GGUF" {
            return Err(Error::Gguf(format!(
                "expected a GGUF file; it begins {magic:?}, not \"GGUF\". A model \
                 host will serve an HTML error page with status 200, which looks \
                 exactly like this."
            )));
        }
        self.u32(); // format version
        self.u64(); // tensor count
        self.u64()
            .ok_or_else(|| Error::Gguf("the header ends before its metadata count".to_owned()))
    }
}
