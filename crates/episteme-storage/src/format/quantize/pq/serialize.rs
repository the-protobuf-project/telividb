//! The on-disk form of a PQ codebook.
//!
//! Separate from training because it is a different concern with a different
//! risk: a codebook arrives inside an archive, so every length it declares is
//! untrusted and must be checked before it is used to allocate or slice.

use super::codebook::{CENTROIDS, PqCodebook};
use crate::error::{Error, Result};

pub(super) const CODEBOOK_MAGIC: [u8; 4] = *b"EPPQ";
pub(super) const CODEBOOK_VERSION: u16 = 1;

/// Bytes before the centroid run: magic, version, dim, m.
const HEADER_BYTES: usize = 14;

impl PqCodebook {
    /// Serialized size in bytes.
    pub fn encoded_len(&self) -> usize {
        HEADER_BYTES + self.centroids.len() * 4
    }

    /// Append the codebook: magic, version, shape, then the centroid run.
    pub fn write_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&CODEBOOK_MAGIC);
        out.extend_from_slice(&CODEBOOK_VERSION.to_le_bytes());
        out.extend_from_slice(&(self.dim as u32).to_le_bytes());
        out.extend_from_slice(&(self.m as u32).to_le_bytes());
        for value in &self.centroids {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }

    /// Parse a codebook, validating every declared length before using it.
    ///
    /// Codebooks arrive inside archives, so this is untrusted input.
    pub fn read_from(bytes: &[u8]) -> Result<Self> {
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
            return Err(Error::InvalidPqShape { dim, m });
        }
        let sub_dim = dim / m;

        let expected = m * CENTROIDS * sub_dim;
        let body = &bytes[HEADER_BYTES..];
        if body.len() < expected * 4 {
            return Err(Error::Truncated {
                what: "pq centroids",
                needed: HEADER_BYTES + expected * 4,
                found: bytes.len(),
            });
        }

        let centroids = body[..expected * 4]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect();

        Ok(Self {
            dim,
            m,
            sub_dim,
            centroids,
        })
    }
}

#[cfg(test)]
#[path = "serialize_test.rs"]
mod tests;
