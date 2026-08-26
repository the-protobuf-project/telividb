//! The scalar reference every other number is compared against.
//!
//! Kept beside the device profile rather than in it because the comparison is
//! the point: an optimization that is not measured against the plain
//! implementation is not measured at all. `FlatIndex` is also the correctness
//! reference for recall (CLAUDE.md rule 8), so it is the one path in the system
//! that should stay deliberately unoptimized.

use crate::{REPEATS, queries_of};
use std::time::Instant;
use telividb_index::adapters::MemoryStore;
use telividb_index::ports::VectorIndex;

/// The scalar-Rust reference, for comparison against ggml's CPU kernels.
///
/// `FlatIndex` is pure Rust over `&[f32]` with no SIMD and no dispatch — the
/// correctness reference every recall number is measured against, and the
/// baseline any optimization has to beat to justify itself.
pub(crate) fn flat_baseline(store: &MemoryStore, dim: usize) {
    use telividb_index::adapters::FlatIndex;

    let index = FlatIndex::new();
    let queries = queries_of(1, dim);
    let query = queries[0].as_slice();

    // One untimed call so the first page-in is not charged to the measurement.
    let _ = index.search(store, query, 10, None);

    let started = Instant::now();
    for _ in 0..REPEATS {
        if index.search(store, query, 10, None).is_err() {
            return;
        }
    }
    let per = started.elapsed().as_secs_f64() / REPEATS as f64;
    println!(
        "flat (scalar rust) : {:.3}ms/query   {:.0} q/s",
        per * 1000.0,
        1.0 / per
    );
}
