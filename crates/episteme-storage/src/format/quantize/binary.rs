//! One bit per dimension.
//!
//! Keeps only the sign of each component, which for normalized vectors
//! preserves enough angular information to rank coarsely at 32× compression.
//!
//! Far too lossy to rank on directly — it is a **first pass**. Scan binary
//! codes to cut millions of candidates to thousands, then rescore those at full
//! precision. Used alone it will return plausible neighbours that are wrong.

/// Sign bits for one row, packed eight to a byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryCodes {
    bits: Vec<u8>,
    dim: usize,
}

impl BinaryCodes {
    /// Pack the sign of each component. Non-negative becomes a set bit.
    ///
    /// Zero maps to one rather than being treated as a third case: it keeps
    /// the encoding total, and a component of exactly zero carries no angular
    /// information either way.
    pub fn encode(vector: &[f32]) -> Self {
        let mut bits = vec![0u8; vector.len().div_ceil(8)];
        for (i, &v) in vector.iter().enumerate() {
            if v >= 0.0 {
                bits[i / 8] |= 1 << (i % 8);
            }
        }
        Self {
            bits,
            dim: vector.len(),
        }
    }

    /// Number of components these bits describe.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// The packed sign bits.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }

    /// Bytes one row occupies, rounded up to whole bytes.
    pub fn encoded_len(dim: usize) -> usize {
        dim.div_ceil(8)
    }

    /// Parse packed bits, returning `None` if the buffer is short.
    pub fn from_bytes(bytes: &[u8], dim: usize) -> Option<Self> {
        if bytes.len() < Self::encoded_len(dim) {
            return None;
        }
        Some(Self {
            bits: bytes[..Self::encoded_len(dim)].to_vec(),
            dim,
        })
    }

    /// Reconstruct a coarse vector of ±1.
    ///
    /// Only the direction of each axis survives; magnitude is gone entirely.
    pub fn decode(&self) -> Vec<f32> {
        (0..self.dim)
            .map(|i| {
                if self.bits[i / 8] & (1 << (i % 8)) != 0 {
                    1.0
                } else {
                    -1.0
                }
            })
            .collect()
    }
}

/// Number of differing bits between two codes.
///
/// Lower means more similar, so this is already a distance. For normalized
/// vectors it tracks angular distance closely enough to prune with.
///
/// Returns `None` if the codes describe different widths — comparing them would
/// silently produce a meaningless number.
pub fn hamming(a: &BinaryCodes, b: &BinaryCodes) -> Option<u32> {
    if a.dim != b.dim {
        return None;
    }
    Some(
        a.bits
            .iter()
            .zip(&b.bits)
            .map(|(x, y)| (x ^ y).count_ones())
            .sum(),
    )
}

#[cfg(test)]
#[path = "binary_test.rs"]
mod tests;
