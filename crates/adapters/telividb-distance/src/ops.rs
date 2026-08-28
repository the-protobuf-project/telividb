//! Vector arithmetic, as methods on the slice itself.
//!
//! A trait rather than free functions because behaviour should hang off the
//! thing it operates on — `a.dot(b)` says what it does to `a`, where
//! `dot(a, b)` leaves the reader to work out which argument is which. Rust's
//! orphan rule forbids an inherent `impl` on `[f32]`, which is not ours, so a
//! trait is how the receiver form is reached across that boundary.
//!
//! Every method here is the scalar fallback. Runtime SIMD dispatch replaces the
//! bodies without changing a single call site, which is the other reason the
//! shape is worth fixing now.

/// Arithmetic every vector supports, whatever the metric above it.
pub trait VectorOps {
    /// Inner product with `other`.
    ///
    /// Zips rather than indexing: the iterator form elides the bounds check per
    /// element and stops at the shorter side, so a length mismatch produces a
    /// short answer rather than a panic. Callers check widths at the boundary
    /// (`Error::DimMismatch`) where the mismatch is still attributable.
    fn dot(&self, other: &[f32]) -> f32;

    /// Squared Euclidean distance to `other`.
    ///
    /// Squared, never rooted. The square root is monotonic so it cannot change
    /// an ordering, and skipping it saves a pass over every candidate — which
    /// is why `Metric::L2` means squared distance throughout this codebase.
    fn l2_squared(&self, other: &[f32]) -> f32;

    /// This vector's Euclidean length.
    fn norm(&self) -> f32;

    /// A unit-length copy.
    ///
    /// A copy rather than in-place, because the common caller is turning a
    /// freshly computed embedding into a stored one and has no use for the
    /// original. See [`VectorOps::normalize`] where the buffer is owned.
    fn normalized(&self) -> Vec<f32>;
}

impl VectorOps for [f32] {
    fn dot(&self, other: &[f32]) -> f32 {
        self.iter().zip(other).map(|(a, b)| a * b).sum()
    }

    fn l2_squared(&self, other: &[f32]) -> f32 {
        self.iter().zip(other).map(|(a, b)| (a - b) * (a - b)).sum()
    }

    fn norm(&self) -> f32 {
        self.dot(self).sqrt()
    }

    fn normalized(&self) -> Vec<f32> {
        let norm = self.norm();
        // A zero vector has no direction to preserve. Returning it unchanged
        // beats dividing by zero, which would poison every later comparison
        // with NaN — and unlike a NaN, a zero vector simply ranks last.
        match norm > 0.0 {
            true => self.iter().map(|v| v / norm).collect(),
            false => self.to_vec(),
        }
    }
}

/// In-place normalisation, for a buffer the caller already owns.
pub trait NormalizeInPlace {
    /// Scale this vector to unit length.
    ///
    /// Cosine is stored normalised and scored as a dot product, so ingest calls
    /// this once rather than paying for a division per comparison at query
    /// time.
    fn normalize(&mut self);
}

impl NormalizeInPlace for [f32] {
    fn normalize(&mut self) {
        let norm = self.norm();
        if norm > 0.0 {
            for value in self.iter_mut() {
                *value /= norm;
            }
        }
    }
}

#[cfg(test)]
#[path = "ops_test.rs"]
mod tests;
