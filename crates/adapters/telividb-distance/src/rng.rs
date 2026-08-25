//! A deterministic generator, so training reproduces exactly.
//!
//! Split from the clustering loop because reproducibility is a *property of the
//! whole crate*, not of k-means: a codebook is baked into every vector encoded
//! against it, so a nondeterministic source anywhere in training means two
//! builds of the same data produce mutually unreadable codes.

/// Deterministic generator, so training reproduces exactly.
pub struct Rng(
    /// Current state. Seeded so training reproduces exactly.
    pub u64,
);

impl Rng {
    /// Next value in the sequence.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `[0, bound)`, or zero when `bound` is zero.
    pub fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % bound as u64) as usize
        }
    }
}
