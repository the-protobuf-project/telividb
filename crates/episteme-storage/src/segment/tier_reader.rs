//! Loading a sealed field's scan tier.
//!
//! Completes the two-tier path: [`SegmentReader`](super::SegmentReader) gives
//! the exact tier, this gives the coarse one, and the search path composes
//! them. Neither knows about the other — which is what lets a field gain or
//! lose a codec without any index changing.

use crate::error::{Error, Result};
use crate::format::{Codec, FIELD_HEADER_BYTES, FieldHeader, quantize::PqCodebook};
use crate::segment::layout::field_dir;
use crate::tier::{BinaryTier, F16Tier, Int8Tier, PqTier};
use episteme_core::ScanTier;
use std::path::Path;

/// Open the scan tier of one field, if it has one.
///
/// Returns `None` when the field was written at full precision only — a normal
/// configuration, not an error, and the caller falls back to searching the
/// exact tier directly.
pub fn open_tier(segment: &Path, field: &str) -> Result<Option<Box<dyn ScanTier>>> {
    let dir = field_dir(segment, field);
    let header_bytes = std::fs::read(dir.join("raw.bin"))?;
    let header = FieldHeader::decode(&header_bytes[..FIELD_HEADER_BYTES.min(header_bytes.len())])?;

    if header.codec == Codec::None {
        return Ok(None);
    }

    let codes = std::fs::read(dir.join("codes.bin"))?;
    let dim = header.dim.get();
    let rows = header.rows as usize;
    let row_bytes = header.codec.row_bytes(dim);

    // A short file means the codes were truncated. Refuse rather than reading
    // whatever happens to follow — every row's offset is computed from this.
    let needed = rows * row_bytes;
    if codes.len() < needed {
        return Err(Error::Truncated {
            what: "codes.bin",
            needed,
            found: codes.len(),
        });
    }

    let present = std::fs::read(dir.join("present.roar")).unwrap_or_default();
    let is_present = |row: usize| {
        present.is_empty() || present.get(row / 8).is_some_and(|b| b & (1 << (row % 8)) != 0)
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
