//! The segment header — segment-level facts, and the schema it was written under.

use crate::error::{Error, Result};
use episteme_core::Fingerprint;

/// Magic bytes at the head of every `header.bin`.
pub const SEGMENT_MAGIC: [u8; 4] = *b"EPSG";
/// Highest segment-header version this build writes and reads.
pub const SEGMENT_VERSION: u16 = 1;

/// `magic(4) version(2) reserved(2) schema_fp(32) rows(8) deleted(8) crc(4)`
/// Fixed encoded size of a segment header.
pub const HEADER_BYTES: usize = 60;

/// Fixed-size preamble of a sealed segment.
///
/// Deliberately carries *no* vector metadata. Dimension, codec, metric and
/// model provenance are per named-vector-field and live in each field's own
/// header, because a point may hold several fields with different dimensions
/// and different models. Anything segment-wide here would be a lie the moment
/// a second field existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentHeader {
    /// Digest of the descriptor set this segment was written under.
    ///
    /// A reader compares it against the collection's current schema. Drift is
    /// refused rather than reconciled: reading columns under the wrong schema
    /// lands values in the wrong places and reports no error at all.
    pub schema_fingerprint: Fingerprint,
    /// Rows written, including tombstoned ones.
    pub rows: u64,
    /// Rows tombstoned since sealing. Bytes remain until compaction.
    pub deleted: u64,
}

impl SegmentHeader {
    /// A header for a freshly sealed segment, with no tombstones.
    pub fn new(schema_fingerprint: Fingerprint, rows: u64) -> Self {
        Self {
            schema_fingerprint,
            rows,
            deleted: 0,
        }
    }

    /// Serialize, appending a checksum over the preceding bytes.
    pub fn encode(&self) -> [u8; HEADER_BYTES] {
        let mut out = [0u8; HEADER_BYTES];
        out[0..4].copy_from_slice(&SEGMENT_MAGIC);
        out[4..6].copy_from_slice(&SEGMENT_VERSION.to_le_bytes());
        // out[6..8] reserved
        out[8..40].copy_from_slice(self.schema_fingerprint.as_bytes());
        out[40..48].copy_from_slice(&self.rows.to_le_bytes());
        out[48..56].copy_from_slice(&self.deleted.to_le_bytes());
        let crc = crc32fast::hash(&out[..56]);
        out[56..60].copy_from_slice(&crc.to_le_bytes());
        out
    }

    /// Parse and validate: magic, version, then checksum, in that order.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_BYTES {
            return Err(Error::Truncated {
                what: "segment header",
                needed: HEADER_BYTES,
                found: bytes.len(),
            });
        }

        let found: [u8; 4] = bytes[0..4].try_into().expect("4 bytes");
        if found != SEGMENT_MAGIC {
            return Err(Error::BadMagic {
                expected: SEGMENT_MAGIC,
                found,
            });
        }

        let version = u16::from_le_bytes(bytes[4..6].try_into().expect("2 bytes"));
        if version > SEGMENT_VERSION {
            return Err(Error::UnsupportedVersion {
                what: "segment",
                found: version,
                supported: SEGMENT_VERSION,
            });
        }

        let expected = u32::from_le_bytes(bytes[56..60].try_into().expect("4 bytes"));
        let computed = crc32fast::hash(&bytes[..56]);
        if expected != computed {
            return Err(Error::Corrupt {
                what: "segment header",
                expected,
                computed,
            });
        }

        let mut fp = [0u8; 32];
        fp.copy_from_slice(&bytes[8..40]);

        Ok(Self {
            schema_fingerprint: Fingerprint::from_bytes(fp),
            rows: u64::from_le_bytes(bytes[40..48].try_into().expect("8 bytes")),
            deleted: u64::from_le_bytes(bytes[48..56].try_into().expect("8 bytes")),
        })
    }

    /// Rows still visible to a reader.
    pub fn live_rows(&self) -> u64 {
        self.rows.saturating_sub(self.deleted)
    }

    /// Refuse a segment written under a different schema.
    ///
    /// An unset fingerprint on either side skips the check — that is the
    /// fixture and pre-schema case, and it means "unknown", never "agrees".
    pub fn check_schema(&self, current: Fingerprint) -> Result<()> {
        if self.schema_fingerprint.is_unset() || current.is_unset() {
            return Ok(());
        }
        if self.schema_fingerprint != current {
            return Err(Error::SchemaDrift {
                segment: self.schema_fingerprint.short(),
                current: current.short(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "header_test.rs"]
mod tests;
