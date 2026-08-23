//! How vectors are stored.

use crate::error::{Error, Result};

/// Precision of the vectors in `raw.bin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DType {
    F32,
    F16,
    BF16,
}

impl DType {
    pub fn bytes_per_element(self) -> usize {
        match self {
            DType::F32 => 4,
            DType::F16 | DType::BF16 => 2,
        }
    }

    pub(crate) fn to_byte(self) -> u8 {
        match self {
            DType::F32 => 0,
            DType::F16 => 1,
            DType::BF16 => 2,
        }
    }

    pub(crate) fn from_byte(value: u8) -> Result<Self> {
        Ok(match value {
            0 => DType::F32,
            1 => DType::F16,
            2 => DType::BF16,
            _ => {
                return Err(Error::UnknownDiscriminant {
                    what: "dtype",
                    value,
                });
            }
        })
    }
}

/// Compression applied to `codes.bin`, the file the wide scan reads.
///
/// This is a *storage* concern rather than an index one — the index sees a
/// vector store, not a codec. That separation is what lets a custom search
/// algorithm work over any of these without knowing which is in play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    /// No second tier; rerank reads full precision directly.
    None,
    F16,
    /// Per-row scale and offset, one byte per dimension.
    Int8,
    /// Product quantization, `m` sub-quantizers of one byte each.
    Pq {
        m: u16,
    },
    /// One bit per dimension.
    Binary,
}

impl Codec {
    /// Bytes one row occupies in `codes.bin`.
    pub fn row_bytes(self, dim: usize) -> usize {
        match self {
            Codec::None => 0,
            Codec::F16 => dim * 2,
            // Eight trailing bytes carry the row's scale and offset.
            Codec::Int8 => dim + 8,
            Codec::Pq { m } => m as usize,
            Codec::Binary => dim.div_ceil(8),
        }
    }

    pub(crate) fn to_bytes(self) -> (u8, u16) {
        match self {
            Codec::None => (0, 0),
            Codec::F16 => (1, 0),
            Codec::Int8 => (2, 0),
            Codec::Pq { m } => (3, m),
            Codec::Binary => (4, 0),
        }
    }

    pub(crate) fn from_bytes(tag: u8, param: u16) -> Result<Self> {
        Ok(match tag {
            0 => Codec::None,
            1 => Codec::F16,
            2 => Codec::Int8,
            3 => Codec::Pq { m: param },
            4 => Codec::Binary,
            value => {
                return Err(Error::UnknownDiscriminant {
                    what: "codec",
                    value,
                });
            }
        })
    }
}
