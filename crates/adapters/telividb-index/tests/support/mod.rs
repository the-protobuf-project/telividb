//! Synthetic corpora for index tests.
//!
//! **Clustered by default, deliberately.** Uniformly random vectors in high
//! dimensions suffer concentration of measure: every pairwise distance
//! converges on the same value, so the true nearest neighbour is barely nearer
//! than the thousandth and every ANN method collapses. At 128 dimensions this
//! index scores 0.41 on uniform data and 0.999 on clustered data of identical
//! size — the difference is the fixture, not the algorithm.
//!
//! Real embeddings have cluster structure and a low intrinsic dimension, so a
//! uniform fixture measures a pathology nobody encounters.

// Each integration test binary compiles this module independently, so anything
// one binary does not use looks dead to that binary. Allowing it here is the
// standard idiom for shared test support; a helper per file would drift.
#![allow(dead_code)]

use telividb_core::{Dim, Metric};
use telividb_index::adapters::MemoryStore;

pub const DIM: usize = 32;
pub const CLUSTERS: usize = 24;

/// SplitMix64, so fixtures need no dependency and reproduce exactly.
pub struct Rng(pub u64);

impl Rng {
    pub fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 40) as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }

    pub fn vector(&mut self) -> Vec<f32> {
        self.vector_of(DIM)
    }

    pub fn vector_of(&mut self, dim: usize) -> Vec<f32> {
        (0..dim).map(|_| self.next_f32()).collect()
    }

    /// Approximately standard normal, by summing uniforms.
    pub fn gaussian(&mut self) -> f32 {
        (0..6).map(|_| self.next_f32()).sum::<f32>() / 6f32.sqrt()
    }

    /// A point near `centre` — the shape real embeddings actually have.
    pub fn near(&mut self, centre: &[f32]) -> Vec<f32> {
        centre.iter().map(|c| c + self.gaussian() * 0.15).collect()
    }
}

/// A clustered corpus with queries drawn from the same distribution.
///
/// Querying a clustered corpus with uniform noise would measure nothing anyone
/// will ever do.
pub fn corpus(rows: usize, metric: Metric, seed: u64) -> (MemoryStore, Vec<Vec<f32>>) {
    let mut rng = Rng(seed);
    let centres: Vec<Vec<f32>> = (0..CLUSTERS).map(|_| rng.vector()).collect();

    let mut store = MemoryStore::new(Dim::new(DIM as u32).unwrap(), metric);
    for row in 0..rows {
        let v = rng.near(&centres[row % CLUSTERS]);
        store.push(&v).unwrap();
    }

    let queries = (0..40).map(|i| rng.near(&centres[i % CLUSTERS])).collect();
    (store, queries)
}
