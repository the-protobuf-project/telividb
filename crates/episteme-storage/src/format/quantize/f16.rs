//! Half precision.
//!
//! Two bytes per component, so exactly half the size of f32 with none of the
//! per-row bookkeeping int8 needs. The trade is different in kind: int8 spends
//! eight bytes a row on a scale and offset to squeeze a *narrow* range into 256
//! steps, while f16 keeps a floating exponent and so handles a wide dynamic
//! range at a coarser relative precision — about 3 decimal digits.
//!
//! For embeddings, whose components cluster tightly around zero, int8 is
//! usually the better ratio. f16 earns its place when values span orders of
//! magnitude, or when a downstream consumer wants floats rather than codes.
//!
//! Uses the `half` crate rather than a hand-rolled conversion: subnormals,
//! round-to-nearest-even, and overflow-to-infinity are exactly the cases a
//! hand-rolled version gets quietly wrong.

use half::f16;

/// One row stored as half-precision components.
///
/// `PartialEq` but not `Eq`: a row holding NaN is not equal to itself, and
/// claiming otherwise would let it be used as a map key that can never be
/// found again.
#[derive(Debug, Clone, PartialEq)]
pub struct F16Row {
    values: Vec<f16>,
}

impl F16Row {
    /// Convert a full-precision row, rounding to nearest even.
    ///
    /// A component beyond f16's range (~65504) saturates to infinity rather
    /// than wrapping. That is IEEE behaviour and it is loud: an infinite score
    /// propagates visibly instead of silently ranking a vector near zero.
    pub fn encode(vector: &[f32]) -> Self {
        Self {
            values: vector.iter().map(|&v| f16::from_f32(v)).collect(),
        }
    }

    pub fn decode(&self) -> Vec<f32> {
        self.values.iter().map(|v| v.to_f32()).collect()
    }

    /// Decode into an existing buffer, avoiding an allocation per row.
    pub fn decode_into(&self, out: &mut [f32]) {
        for (slot, value) in out.iter_mut().zip(self.values.iter()) {
            *slot = value.to_f32();
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Largest value f16 represents. Beyond this, encoding saturates.
    pub fn max_finite() -> f32 {
        f16::MAX.to_f32()
    }

    /// Smallest positive value that survives encoding, including subnormals.
    ///
    /// Anything smaller flushes to zero — which for a normalized embedding
    /// component is harmless, and for an unnormalized one may not be.
    pub fn min_positive() -> f32 {
        f16::from_bits(1).to_f32()
    }

    pub fn encoded_len(dim: usize) -> usize {
        dim * 2
    }

    /// Append little-endian bit patterns.
    pub fn write_to(&self, out: &mut Vec<u8>) {
        for value in &self.values {
            out.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }

    pub fn read_from(bytes: &[u8], dim: usize) -> Option<Self> {
        if bytes.len() < Self::encoded_len(dim) {
            return None;
        }
        let values = bytes[..Self::encoded_len(dim)]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| f16::from_bits(u16::from_le_bytes(*c)))
            .collect();
        Some(Self { values })
    }
}

#[cfg(test)]
#[path = "f16_test.rs"]
mod tests;
