//! Loading a sealed field's scan tier.
//!
//! Completes the two-tier path: [`SegmentReader`](super::SegmentReader) gives
//! the exact tier, this gives the coarse one, and the search path composes
//! them. Neither knows about the other — which is what lets a field gain or
//! lose a codec without any index changing.

use crate::error::{Error, Result};
use crate::format::{Codec, FIELD_HEADER_BYTES, FieldHeader, quantize::PqCodebook};
use crate::ports::{BlockReader, FileBlockReader};
use crate::segment::layout::field_dir;
use crate::tier::{BinaryTier, F16Tier, Int8Tier, PqTier};
use std::path::Path;
use telividb_core::ScanTier;

/// Open the scan tier of one field, if it has one.
///
/// Returns `None` when the field was written at full precision only — a normal
/// configuration, not an error, and the caller falls back to searching the
/// exact tier directly.
pub fn open_tier(segment: &Path, field: &str) -> Result<Option<Box<dyn ScanTier>>> {
    let dir = field_dir(segment, field);

    // Only the header, not the file. `raw.bin` holds every vector at full
    // precision — reading all of it to parse thirty-two bytes and then dropping
    // it violates invariant 3 outright, and on a large field it is gigabytes of
    // page cache churn per open.
    let raw = FileBlockReader::open(dir.join("raw.bin"))?;
    let mut header_bytes = [0u8; FIELD_HEADER_BYTES];
    raw.read_exact_at(0, &mut header_bytes)?;
    let header = FieldHeader::decode(&header_bytes)?;

    if header.codec == Codec::None {
        return Ok(None);
    }

    let dim = header.dim.get();
    let rows = header.rows as usize;
    let row_bytes = header.codec.row_bytes(dim);

    // The codes *are* the scan tier and every row is touched by a scan, so
    // unlike `raw.bin` they are read whole — but through the port, and only
    // after the declared length is checked against the file. Refuse a short
    // file rather than reading whatever happens to follow: every row's offset
    // is computed from this.
    let codes_reader = FileBlockReader::open(dir.join("codes.bin"))?;
    let needed = rows.checked_mul(row_bytes).ok_or(Error::Truncated {
        what: "codes.bin",
        needed: usize::MAX,
        found: 0,
    })?;
    if (codes_reader.len() as usize) < needed {
        return Err(Error::Truncated {
            what: "codes.bin",
            needed,
            found: codes_reader.len() as usize,
        });
    }
    let mut codes = vec![0u8; needed];
    codes_reader.read_exact_at(0, &mut codes)?;

    let present = read_present(&dir, rows);
    let is_present = |row: usize| {
        present.is_empty()
            || present
                .get(row / 8)
                .is_some_and(|b| b & (1 << (row % 8)) != 0)
    };

    let tier: Box<dyn ScanTier> = match header.codec {
        Codec::None => return Ok(None),
        Codec::F16 => Box::new(F16Tier::from_codes(&codes, dim, rows, &is_present)?),
        Codec::Int8 => Box::new(Int8Tier::from_codes(&codes, dim, rows, &is_present)?),
        Codec::Binary => Box::new(BinaryTier::from_codes(&codes, dim, rows, &is_present)?),
        Codec::Pq { .. } => {
            let book = PqCodebook::read_from(&std::fs::read(dir.join("codebook.pq"))?)?;
            Box::new(PqTier::from_codes(&codes, book, rows, &is_present)?)
        }
    };
    Ok(Some(tier))
}

/// The presence bitmap for a field, or an empty vector if it has none.
///
/// An absent file is treated as "every row present", which is what a field
/// written before the bitmap existed means. A short one is *not* padded: the
/// caller's bounds check turns a missing byte into "absent", which is the
/// fail-closed direction — a row wrongly reported present would be scored from
/// zeroed bytes and rank as a real result.
fn read_present(dir: &std::path::Path, rows: usize) -> Vec<u8> {
    let path = dir.join("present.roar");
    let Ok(reader) = FileBlockReader::open(&path) else {
        return Vec::new();
    };
    let len = (reader.len() as usize).min(rows.div_ceil(8));
    let mut bytes = vec![0u8; len];
    if reader.read_exact_at(0, &mut bytes).is_err() {
        return Vec::new();
    }
    bytes
}
