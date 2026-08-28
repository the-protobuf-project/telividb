//! Product quantization: a vector as `m` small codes.
//!
//! **Why it lives here rather than in storage.** Two callers need it and
//! neither may depend on the other: the scan tier in `telividb-storage`
//! compresses stored rows, and IVF-PQ in `telividb-index` compresses the rows
//! inside each inverted list. `telividb-distance` is the crate both already
//! depend on — and product quantization is a distance construction, not a file
//! format: scoring a code against a query is asymmetric distance computation,
//! which is exactly the kind of kernel this crate exists to hold.
//!
//! The *serialization* of a trained codebook stays in storage, where the
//! versioned on-disk format belongs.

mod adc;
mod codebook;
mod encode;

pub use codebook::{CENTROIDS, PqCodebook, PqParams};

#[cfg(test)]
#[path = "training_test.rs"]
mod training_tests;
