//! A clustering run, configured as a value.
//!
//! Split from `cluster.rs` so that file is Lloyd's algorithm and this is the
//! API over it. The separation matters because callers only ever touch this
//! side — the loop below it is an implementation detail that two different
//! consumers (PQ subspaces and IVF lists) share without knowing about each
//! other.

use crate::cluster;

/// A configured clustering run.
///
/// A value rather than five positional arguments: `train(points, dim, k, iters,
/// seed)` gives a reader no way to tell `k` from `iterations` at the call site,
/// and the two are both small integers. Built once and reused, so the settings
/// travel with the thing that uses them.
#[derive(Debug, Clone, Copy)]
pub struct KMeans {
    dim: usize,
    k: usize,
    iterations: usize,
    seed: u64,
}

impl KMeans {
    /// Cluster `dim`-wide vectors into `k` centroids.
    pub fn new(dim: usize, k: usize) -> Self {
        Self {
            dim,
            k,
            // Few by default: Lloyd converges quickly at the sizes here, and
            // each extra pass is a full sweep of the training set.
            iterations: 12,
            seed: 0x5EED_1F5F,
        }
    }

    /// Run for a different number of Lloyd iterations.
    pub fn iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    /// Seed the sampling and initialisation differently.
    ///
    /// Seeded rather than random because a codebook is baked into every vector
    /// encoded against it: a nondeterministic trainer means two builds of the
    /// same data produce mutually unreadable codes.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// How many centroids this run produces.
    pub fn k(self) -> usize {
        self.k
    }

    /// Train, returning centroids laid out contiguously as `k * dim` floats.
    pub fn train(self, points: &[&[f32]]) -> Vec<f32> {
        cluster::train_impl(points, self.dim, self.k, self.iterations, self.seed)
    }

    /// Which of `centroids` is nearest `point`.
    ///
    /// On the same value as `train` because assignment must use the measure
    /// training used — clustering under one and assigning under another puts
    /// rows in lists whose centroid does not describe them.
    pub fn assign(self, point: &[f32], centroids: &[f32]) -> usize {
        cluster::nearest_centroid(point, centroids, self.dim)
    }
}
