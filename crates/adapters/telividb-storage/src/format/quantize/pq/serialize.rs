//! The on-disk form of a PQ codebook.
//!
//! Separate from training because it is a different concern with a different
//! risk: a codebook arrives inside an archive, so every length it declares is
//! untrusted and must be checked before it is used to allocate or slice.

use crate::error::{Error, Result};
use telividb_distance::pq::{CENTROIDS, PqCodebook};

pub(super) const CODEBOOK_MAGIC: [u8; 4] = *b"EPPQ";
pub(super) const CODEBOOK_VERSION: u16 = 1;

/// Bytes before the centroid run: magic, version, dim, m.
const HEADER_BYTES: usize = 14;

/// Bytes [`encode_codebook`] will write.
pub fn encoded_len(book: &PqCodebook) -> usize {
    HEADER_BYTES + book.centroids().len() * 4
}

/// Append the codebook's bytes to `out`.
pub fn encode_codebook(book: &PqCodebook, out: &mut Vec<u8>) {
    out.extend_from_slice(&CODEBOOK_MAGIC);
    out.extend_from_slice(&CODEBOOK_VERSION.to_le_bytes());
    out.extend_from_slice(&(book.dim() as u32).to_le_bytes());
    out.extend_from_slice(&(book.m() as u32).to_le_bytes());
    for value in book.centroids() {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

/// Parse a codebook, validating every declared length before using it.
///
/// Codebooks arrive inside archives, so this is untrusted input.
/// Read a codebook back, refusing anything this build cannot interpret.
pub fn decode_codebook(bytes: &[u8]) -> Result<PqCodebook> {
    if bytes.len() < HEADER_BYTES {
        return Err(Error::Truncated {
            what: "pq codebook",
            needed: HEADER_BYTES,
            found: bytes.len(),
        });
    }
    let magic: [u8; 4] = bytes[0..4].try_into().expect("4 bytes");
    if magic != CODEBOOK_MAGIC {
        return Err(Error::BadMagic {
            expected: CODEBOOK_MAGIC,
            found: magic,
        });
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().expect("2 bytes"));
    if version > CODEBOOK_VERSION {
        return Err(Error::UnsupportedVersion {
            what: "pq codebook",
            found: version,
            supported: CODEBOOK_VERSION,
        });
    }

    let dim = u32::from_le_bytes(bytes[6..10].try_into().expect("4 bytes")) as usize;
    let m = u32::from_le_bytes(bytes[10..14].try_into().expect("4 bytes")) as usize;
    if m == 0 || dim == 0 || !dim.is_multiple_of(m) {
        return Err(telividb_core::Error::InvalidPqShape { dim, m }.into());
    }
    let sub_dim = dim / m;

    // Checked, in a function whose whole contract is validating declared
    // lengths before they are used. `dim` and `m` come from the file, so on
    // a 32-bit target the product overflows well inside `u32` — and an
    // overflowed length wraps to something small that passes the check
    // below, then indexes far past the buffer.
    let expected = m
        .checked_mul(CENTROIDS)
        .and_then(|n| n.checked_mul(sub_dim))
        .ok_or(Error::from(telividb_core::Error::InvalidPqShape { dim, m }))?;
    let needed_bytes = expected
        .checked_mul(4)
        .and_then(|n| n.checked_add(HEADER_BYTES))
        .ok_or(Error::from(telividb_core::Error::InvalidPqShape { dim, m }))?;

    let body = &bytes[HEADER_BYTES..];
    if bytes.len() < needed_bytes {
        return Err(Error::Truncated {
            what: "pq centroids",
            needed: needed_bytes,
            found: bytes.len(),
        });
    }

    let centroids = body[..expected * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect();

    // `sub_dim` is not passed: the codebook derives it from `dim / m`, so
    // handing it a second copy would be two answers to one question.
    Ok(PqCodebook::from_parts(dim, m, centroids)?)
}

#[cfg(test)]
#[path = "serialize_test.rs"]
mod tests;
