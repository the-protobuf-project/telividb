//! Encoding a vector to codes, and reconstructing it back.
//!
//! Split from `codebook.rs` because that file is about *training* a codebook
//! and this is about using one. Reconstruction lives here too: it is the
//! inverse of encoding, and the only way to see what quantization cost.

use super::codebook::{CENTROIDS, PqCodebook};
use crate::kmeans;
use telividb_core::{Error, Result};

impl PqCodebook {
    /// Encode one vector to `m` bytes.
    pub fn encode(&self, vector: &[f32]) -> Result<Vec<u8>> {
        if vector.len() != self.dim {
            return Err(Error::PqDimMismatch {
                expected: self.dim,
                actual: vector.len(),
            });
        }
        Ok((0..self.m)
            .map(|sub| {
                let start = sub * self.sub_dim;
                let point = &vector[start..start + self.sub_dim];
                kmeans::KMeans::new(self.sub_dim, CENTROIDS).assign(point, self.subspace(sub)) as u8
            })
            .collect())
    }

    /// Reconstruct an approximate vector from its codes.
    pub fn decode(&self, codes: &[u8]) -> Result<Vec<f32>> {
        if codes.len() != self.m {
            return Err(Error::PqDimMismatch {
                expected: self.m,
                actual: codes.len(),
            });
        }
        let mut out = Vec::with_capacity(self.dim);
        for (sub, &code) in codes.iter().enumerate() {
            let start = code as usize * self.sub_dim;
            out.extend_from_slice(&self.subspace(sub)[start..start + self.sub_dim]);
        }
        Ok(out)
    }
}
