//! Product quantization: a codebook per subspace.
//!
//! The vector is split into `m` contiguous subvectors, each quantized against
//! its own 256-entry codebook trained by k-means. A row becomes `m` bytes —
//! one code per subspace — regardless of how wide it started.
//!
//! Why it beats scalar quantization at high ratios: int8 spends a byte per
//! *dimension*, so 768 dims cost 768 bytes. PQ spends a byte per *subspace*, so
//! the same vector at `m = 96` costs 96 bytes — 32× compression — while still
//! reconstructing each 8-dimensional chunk from a centroid fitted to the actual
//! data distribution, rather than from a uniform grid.
//!
//! The cost is training. A codebook must be fitted before anything can be
//! encoded, it is baked into every code written against it, and a code is
//! meaningless without exactly the codebook that produced it — which is why the
//! codebook lives in the segment beside the codes and never in shared state.

use super::kmeans;
use crate::error::{Error, Result};

/// Entries per subspace codebook. One byte per code, so 256.
pub const CENTROIDS: usize = 256;

/// How a vector is divided into subspaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PqParams {
    /// Number of subspaces, and therefore bytes per encoded row.
    pub m: usize,
    /// Lloyd iterations during training.
    pub iterations: usize,
    /// Seed, so a codebook reproduces exactly.
    pub seed: u64,
}

impl Default for PqParams {
    fn default() -> Self {
        Self {
            m: 8,
            iterations: 25,
            seed: 0x9e37_79b9_7f4a_7c15,
        }
    }
}

/// Trained centroids for every subspace.
#[derive(Debug, Clone, PartialEq)]
pub struct PqCodebook {
    /// Full vector width this codebook was trained for.
    pub(super) dim: usize,
    /// Subspaces. `dim` must divide evenly by this.
    pub(super) m: usize,
    /// Width of one subspace.
    pub(super) sub_dim: usize,
    /// `m` codebooks of `CENTROIDS * sub_dim` floats each.
    pub(super) centroids: Vec<f32>,
}

impl PqCodebook {
    /// Fit a codebook to `training` vectors.
    ///
    /// Rejects a width that does not divide evenly rather than padding: a
    /// silent pad would make the last subspace partly meaningless and the
    /// resulting recall loss would be very hard to attribute.
    pub fn train(training: &[&[f32]], dim: usize, params: PqParams) -> Result<Self> {
        if params.m == 0 || dim == 0 || !dim.is_multiple_of(params.m) {
            return Err(Error::InvalidPqShape { dim, m: params.m });
        }
        let sub_dim = dim / params.m;
        let mut centroids = Vec::with_capacity(params.m * CENTROIDS * sub_dim);

        for sub in 0..params.m {
            let start = sub * sub_dim;
            let slices: Vec<&[f32]> = training
                .iter()
                .filter(|v| v.len() >= start + sub_dim)
                .map(|v| &v[start..start + sub_dim])
                .collect();

            // Each subspace gets its own seed, or every codebook would be
            // seeded identically and the first pick would correlate across
            // subspaces.
            centroids.extend(kmeans::train(
                &slices,
                sub_dim,
                CENTROIDS,
                params.iterations,
                params.seed.wrapping_add(sub as u64),
            ));
        }

        Ok(Self {
            dim,
            m: params.m,
            sub_dim,
            centroids,
        })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn m(&self) -> usize {
        self.m
    }

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
                kmeans::nearest_centroid(point, self.subspace(sub), self.sub_dim) as u8
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

    /// One centroid of one subspace.
    ///
    /// Exposed so a scan tier can build a distance table without copying the
    /// whole codebook or reaching into its layout.
    pub fn centroid(&self, sub: usize, index: usize) -> &[f32] {
        let start = index * self.sub_dim;
        &self.subspace(sub)[start..start + self.sub_dim]
    }

    /// Centroids belonging to one subspace.
    fn subspace(&self, sub: usize) -> &[f32] {
        let span = CENTROIDS * self.sub_dim;
        &self.centroids[sub * span..(sub + 1) * span]
    }
}

#[cfg(test)]
#[path = "codebook_test.rs"]
mod tests;
