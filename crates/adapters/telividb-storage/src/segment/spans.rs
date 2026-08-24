//! Temporal spans on disk.
//!
//! A 90-minute recording is not one embedding — retrieval returns *a moment*,
//! so a media point carries the interval it covers. Spans live in their own
//! fixed-stride file rather than in the payload because they are queried
//! structurally: overlap, containment and proximity are range predicates, and a
//! columnar payload scan is the wrong shape for those.
//!
//! Sixteen bytes a row, two little-endian `u64` milliseconds. Fixed stride, so
//! a row's offset is its ordinal times the stride — the same property that
//! makes `raw.bin` mappable.

use crate::error::{Error, Result};
use telividb_core::Span;

/// Bytes one span occupies: a start and an end.
pub const SPAN_BYTES: usize = 16;

/// A row with no span, written as two `u64::MAX` sentinels.
///
/// A sentinel rather than a presence bitmap because a span file is already
/// optional per field, and an all-ones pair cannot be a valid interval — the
/// end would precede the start by the widest possible margin.
const ABSENT: u64 = u64::MAX;

/// Serialize spans, one entry per row.
///
/// `None` marks a row that carries no span, which is normal: a text chunk has
/// no temporal extent while the transcript segment beside it does.
pub fn encode(spans: &[Option<Span>]) -> Vec<u8> {
    let mut out = Vec::with_capacity(spans.len() * SPAN_BYTES);
    for span in spans {
        match span {
            Some(s) => {
                out.extend_from_slice(&s.start_ms().to_le_bytes());
                out.extend_from_slice(&s.end_ms().to_le_bytes());
            }
            None => {
                out.extend_from_slice(&ABSENT.to_le_bytes());
                out.extend_from_slice(&ABSENT.to_le_bytes());
            }
        }
    }
    out
}

/// Parse a span file of `rows` entries.
///
/// A short file is refused rather than truncated silently: every offset after
/// the missing row would be wrong, and the error would surface as mismatched
/// timestamps far from here.
pub fn decode(bytes: &[u8], rows: usize) -> Result<Vec<Option<Span>>> {
    let needed = rows * SPAN_BYTES;
    if bytes.len() < needed {
        return Err(Error::Truncated {
            what: "spans.bin",
            needed,
            found: bytes.len(),
        });
    }

    bytes[..needed]
        .as_chunks::<SPAN_BYTES>()
        .0
        .iter()
        .map(|chunk| {
            let start = u64::from_le_bytes(chunk[0..8].try_into().expect("8 bytes"));
            let end = u64::from_le_bytes(chunk[8..16].try_into().expect("8 bytes"));
            if start == ABSENT && end == ABSENT {
                return Ok(None);
            }
            Span::new(start, end).map(Some).map_err(Error::from)
        })
        .collect()
}

/// Byte offset of one row's span.
///
/// Fixed stride means a single span can be read without parsing the file, which
/// is what a span predicate needs once filtering exists.
pub fn offset_of(row: usize) -> usize {
    row * SPAN_BYTES
}

#[cfg(test)]
#[path = "spans_test.rs"]
mod tests;
