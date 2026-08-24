//! A tiny deterministic generator for level assignment.
//!
//! Hand-rolled rather than a dependency because the requirement is narrow:
//! reproducible levels from a seed. Nothing here is cryptographic, and it must
//! never be used where that matters.
//!
//! Determinism is the point. A recall regression has to be attributable to a
//! code change, not to which levels the nodes happened to land on — so the same
//! seed and the same insertion order must always produce the same graph.

/// SplitMix64. Small, fast, and good enough for choosing levels.
#[derive(Debug, Clone)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    /// Start the sequence at `seed`.
    ///
    /// The seed is what makes a build reproducible: the same input must yield
    /// the same level assignment, or a recall change cannot be attributed to a
    /// code change rather than to luck.
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The next value in the sequence.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `(0, 1)`. Never returns zero, so `ln` of it stays finite.
    pub fn next_f64(&mut self) -> f64 {
        // 53 significant bits, shifted off zero.
        let bits = self.next_u64() >> 11;
        (bits as f64 + 1.0) / (((1u64 << 53) as f64) + 1.0)
    }

    /// Draw a level from the exponential distribution HNSW uses.
    ///
    /// `floor(-ln(U) * factor)`, which puts roughly `1/m` of nodes on each
    /// successive layer.
    pub fn level(&mut self, factor: f64) -> usize {
        (-self.next_f64().ln() * factor).floor() as usize
    }
}

#[cfg(test)]
#[path = "rng_test.rs"]
mod tests;
