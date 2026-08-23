//! Per named-vector-field metadata.
//!
//! One of these sits in `vectors/<field>/`, beside that field's `raw.bin`,
//! `codes.bin`, index and presence bitmap. It exists because a point carries
//! several vector fields with different dimensions, metrics, codecs and models
//! — so none of those facts can live in the segment header.

use super::{Codec, DType};
use crate::error::{Error, Result};
use episteme_core::{Dim, Fingerprint, Metric};

pub const FIELD_MAGIC: [u8; 4] = *b"EPFD";
pub const FIELD_VERSION: u16 = 1;

/// `magic(4) version(2) dim(4) dtype(1) codec_tag(1) codec_param(2) metric(1)
///  reserved(1) model_fp(32) rows(8) crc(4)`
pub const FIELD_HEADER_BYTES: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldHeader {
    pub dim: Dim,
    pub dtype: DType,
    pub codec: Codec,
    pub metric: Metric,
    /// Digest of the model file that produced every vector in this field.
    ///
    /// Mixing models within one field is the failure that never announces
    /// itself: no error, no crash, just neighbours that are plausible and wrong.
    pub model_fingerprint: Fingerprint,
    pub rows: u64,
}

impl FieldHeader {
    pub fn encode(&self) -> [u8; FIELD_HEADER_BYTES] {
        let mut out = [0u8; FIELD_HEADER_BYTES];
        let (codec_tag, codec_param) = self.codec.to_bytes();

        out[0..4].copy_from_slice(&FIELD_MAGIC);
        out[4..6].copy_from_slice(&FIELD_VERSION.to_le_bytes());
        out[6..10].copy_from_slice(&(self.dim.get() as u32).to_le_bytes());
        out[10] = self.dtype.to_byte();
        out[11] = codec_tag;
        out[12..14].copy_from_slice(&codec_param.to_le_bytes());
        out[14] = metric_to_byte(self.metric);
        // out[15] reserved
        out[16..48].copy_from_slice(self.model_fingerprint.as_bytes());
        out[48..56].copy_from_slice(&self.rows.to_le_bytes());
        let crc = crc32fast::hash(&out[..56]);
        out[56..60].copy_from_slice(&crc.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < FIELD_HEADER_BYTES {
            return Err(Error::Truncated {
                what: "field header",
                needed: FIELD_HEADER_BYTES,
                found: bytes.len(),
            });
        }

        let found: [u8; 4] = bytes[0..4].try_into().expect("4 bytes");
        if found != FIELD_MAGIC {
            return Err(Error::BadMagic {
                expected: FIELD_MAGIC,
                found,
            });
        }

        let version = u16::from_le_bytes(bytes[4..6].try_into().expect("2 bytes"));
        if version > FIELD_VERSION {
            return Err(Error::UnsupportedVersion {
                what: "field",
                found: version,
                supported: FIELD_VERSION,
            });
        }

        let expected = u32::from_le_bytes(bytes[56..60].try_into().expect("4 bytes"));
        let computed = crc32fast::hash(&bytes[..56]);
        if expected != computed {
            return Err(Error::Corrupt {
                what: "field header",
                expected,
                computed,
            });
        }

        let mut fp = [0u8; 32];
        fp.copy_from_slice(&bytes[16..48]);

        Ok(Self {
            dim: Dim::new(u32::from_le_bytes(
                bytes[6..10].try_into().expect("4 bytes"),
            ))?,
            dtype: DType::from_byte(bytes[10])?,
            codec: Codec::from_bytes(
                bytes[11],
                u16::from_le_bytes(bytes[12..14].try_into().expect("2 bytes")),
            )?,
            metric: metric_from_byte(bytes[14])?,
            model_fingerprint: Fingerprint::from_bytes(fp),
            rows: u64::from_le_bytes(bytes[48..56].try_into().expect("8 bytes")),
        })
    }

    /// Bytes one row occupies in `raw.bin`.
    pub fn raw_row_bytes(&self) -> usize {
        self.dim.get() * self.dtype.bytes_per_element()
    }

    /// Bytes one row occupies in `codes.bin`; zero when there is no scan tier.
    pub fn codes_row_bytes(&self) -> usize {
        self.codec.row_bytes(self.dim.get())
    }

    /// Refuse vectors produced by a different model.
    pub fn check_model(&self, current: Fingerprint) -> Result<()> {
        if self.model_fingerprint.is_unset() || current.is_unset() {
            return Ok(());
        }
        if self.model_fingerprint != current {
            return Err(Error::ModelDrift {
                segment: self.model_fingerprint.short(),
                current: current.short(),
            });
        }
        Ok(())
    }
}

fn metric_to_byte(metric: Metric) -> u8 {
    match metric {
        Metric::Dot => 0,
        Metric::L2 => 1,
        Metric::Cosine => 2,
    }
}

fn metric_from_byte(value: u8) -> Result<Metric> {
    Ok(match value {
        0 => Metric::Dot,
        1 => Metric::L2,
        2 => Metric::Cosine,
        v => Err(Error::UnknownDiscriminant {
            what: "metric",
            value: v,
        })?,
    })
}

#[cfg(test)]
#[path = "field_header_test.rs"]
mod tests;
