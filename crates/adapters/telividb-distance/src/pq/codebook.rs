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

use crate::kmeans;
use telividb_core::{Error, Result};

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
    ///
    /// # Errors
    ///
    /// Also rejects a training set smaller than [`CENTROIDS`]. There is no
    /// useful codebook to be had from fewer vectors than centroids — at zero,
    /// seeding returns zeros and the update loop never runs, so every row
    /// encodes to code 0 and the tier ranks nothing. That failure is total,
    /// silent, and indistinguishable from a working index until someone
    /// measures recall.
    pub fn train(training: &[&[f32]], dim: usize, params: PqParams) -> Result<Self> {
        if params.m == 0 || dim == 0 || !dim.is_multiple_of(params.m) {
            return Err(Error::InvalidPqShape { dim, m: params.m });
        }
        if training.len() < CENTROIDS {
            return Err(Error::PqTrainingTooSmall {
                needed: CENTROIDS,
                found: training.len(),
            });
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
            centroids.extend(
                kmeans::KMeans::new(sub_dim, CENTROIDS)
                    .iterations(params.iterations)
                    .seed(params.seed.wrapping_add(sub as u64))
                    .train(&slices),
            );
        }

        Ok(Self {
            dim,
            m: params.m,
            sub_dim,
            centroids,
        })
    }

    /// Every centroid, laid out contiguously as `m * CENTROIDS * sub_dim`.
    ///
    /// Exposed for serialization, which lives in `telividb-storage` because a
    /// codebook's *bytes* carry magic and a format version (rule 4) while its
    /// arithmetic does not.
    pub fn centroids(&self) -> &[f32] {
        &self.centroids
    }

    /// Rebuild a codebook from bytes a reader has already validated.
    ///
    /// `sub_dim` is derived rather than taken: it is `dim / m` by definition,
    /// and accepting a third number would allow a caller to pass one that
    /// disagrees — which would slice every centroid at the wrong stride.
    pub fn from_parts(dim: usize, m: usize, centroids: Vec<f32>) -> Result<Self> {
        check_shape(dim, m)?;
        Ok(Self {
            sub_dim: dim / m,
            dim,
            m,
            centroids,
        })
    }

    /// Full vector width this codebook was trained for.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Subspaces, and therefore bytes per encoded row.
    pub fn m(&self) -> usize {
        self.m
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
    pub(super) fn subspace(&self, sub: usize) -> &[f32] {
        let span = CENTROIDS * self.sub_dim;
        &self.centroids[sub * span..(sub + 1) * span]
    }
}

#[cfg(test)]
#[path = "codebook_test.rs"]
mod tests;

/// Refuse a subspace count that cannot divide the vector evenly.
///
/// Padding the final subspace instead would make part of it meaningless, and
/// the resulting recall loss is very hard to attribute back to here.
fn check_shape(dim: usize, m: usize) -> Result<()> {
    match m == 0 || !dim.is_multiple_of(m) {
        true => Err(Error::InvalidPqShape { dim, m }),
        false => Ok(()),
    }
}
