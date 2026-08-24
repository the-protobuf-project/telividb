//! Scalar quantization to one byte per dimension.
//!
//! Each row carries its own scale and offset, so a row whose components span a
//! narrow range keeps that range's full resolution instead of being squeezed
//! into a corpus-wide scale. Per-row costs eight bytes and is worth it:
//! embedding components vary in range far more between rows than within them.

/// One quantized row: `dim` codes followed by `scale` and `offset`.
#[derive(Debug, Clone, PartialEq)]
pub struct Int8Row {
    /// One byte per dimension, mapping `[offset, offset + 255 * scale]`.
    pub codes: Vec<u8>,
    /// Width of one quantization step. Zero for a constant row.
    pub scale: f32,
    /// Value that code zero decodes to — the row's minimum.
    pub offset: f32,
}

impl Int8Row {
    /// Quantize a full-precision row.
    ///
    /// Maps `[min, max]` onto `[0, 255]`. A constant row — every component
    /// equal, which happens for zero vectors and for padding — gets a zero
    /// scale and decodes back to exactly that constant rather than dividing by
    /// zero.
    pub fn encode(vector: &[f32]) -> Self {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for &v in vector {
            min = min.min(v);
            max = max.max(v);
        }
        if !min.is_finite() || !max.is_finite() {
            min = 0.0;
            max = 0.0;
        }

        let span = max - min;
        let scale = if span > 0.0 { span / 255.0 } else { 0.0 };

        let codes = vector
            .iter()
            .map(|&v| {
                if scale == 0.0 {
                    0u8
                } else {
                    // `round` rather than truncate: truncation biases every
                    // component downward, which shifts the whole vector and
                    // shows up as a systematic ranking error.
                    (((v - min) / scale).round()).clamp(0.0, 255.0) as u8
                }
            })
            .collect();

        Self {
            codes,
            scale,
            offset: min,
        }
    }

    /// Reconstruct the approximate original.
    pub fn decode(&self) -> Vec<f32> {
        self.codes
            .iter()
            .map(|&c| c as f32 * self.scale + self.offset)
            .collect()
    }

    /// Decode into an existing buffer, avoiding an allocation per row.
    ///
    /// The rerank path decodes one row per candidate, so allocating there would
    /// dominate the work being measured.
    pub fn decode_into(&self, out: &mut [f32]) {
        for (slot, &code) in out.iter_mut().zip(self.codes.iter()) {
            *slot = code as f32 * self.scale + self.offset;
        }
    }

    /// Largest possible reconstruction error for any component.
    ///
    /// Half a quantization step, by construction. Useful for deciding whether a
    /// candidate's quantized score could possibly beat another's once both are
    /// rescored at full precision.
    pub fn max_error(&self) -> f32 {
        self.scale / 2.0
    }

    /// Serialized length for a row of `dim` components.
    pub fn encoded_len(dim: usize) -> usize {
        dim + 8
    }

    /// Append `codes`, then `scale` and `offset`, little-endian.
    pub fn write_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.codes);
        out.extend_from_slice(&self.scale.to_le_bytes());
        out.extend_from_slice(&self.offset.to_le_bytes());
    }

    /// Parse a row of `dim` components from `bytes`.
    pub fn read_from(bytes: &[u8], dim: usize) -> Option<Self> {
        if bytes.len() < Self::encoded_len(dim) {
            return None;
        }
        let (codes, tail) = bytes.split_at(dim);
        Some(Self {
            codes: codes.to_vec(),
            scale: f32::from_le_bytes(tail[0..4].try_into().ok()?),
            offset: f32::from_le_bytes(tail[4..8].try_into().ok()?),
        })
    }
}

#[cfg(test)]
#[path = "int8_test.rs"]
mod tests;
