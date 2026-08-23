//! Product quantization.

mod codebook;
mod kmeans;
mod serialize;

pub use codebook::{CENTROIDS, PqCodebook, PqParams};
