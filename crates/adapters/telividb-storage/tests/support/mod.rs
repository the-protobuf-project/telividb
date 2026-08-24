//! Clustered corpora for storage integration tests.
//!
//! Clustered rather than uniform, matching real embeddings: uniform noise has
//! no structure for a codebook to capture, so it would understate PQ badly and
//! overstate how much any codec loses.

#![allow(dead_code)]

use telividb_core::{Dim, Metric};
use telividb_index::adapters::MemoryStore;

pub const DIM: usize = 64;
pub const ROWS: usize = 2_000;

pub struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 40) as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }
}

pub fn corpus() -> (MemoryStore, Vec<Vec<f32>>) {
    let mut rng = Rng(11);
    let centres: Vec<Vec<f32>> = (0..24)
        .map(|_| (0..DIM).map(|_| rng.next()).collect())
        .collect();

    let mut store = MemoryStore::new(Dim::new(DIM as u32).unwrap(), Metric::Cosine);
    for i in 0..ROWS {
        let v: Vec<f32> = centres[i % centres.len()]
            .iter()
            .map(|c| c + rng.next() * 0.12)
            .collect();
        store.push(&v).unwrap();
    }

    let queries = (0..30)
        .map(|i| {
            centres[i % centres.len()]
                .iter()
                .map(|c| c + rng.next() * 0.12)
                .collect()
        })
        .collect();
    (store, queries)
}
