//! Product quantization.

mod codebook;
mod kmeans;
mod serialize;

pub use codebook::{CENTROIDS, PqCodebook, PqParams};

#[cfg(test)]
#[path = "training_test.rs"]
mod training_tests;
