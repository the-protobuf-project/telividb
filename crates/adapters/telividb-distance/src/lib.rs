//! Distance kernels.
//!
//! Rudimentary for now: scalar only. The dispatch seam exists so AVX2 / AVX-512
//! / NEON implementations slot in behind it without touching callers, and so
//! the scalar path stays as the always-correct reference the SIMD versions are
//! tested against. See CLAUDE.md invariant 7.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod cluster;
pub mod kmeans;
mod ops;
pub mod pq;
pub mod rng;
mod scoring;

pub use ops::{NormalizeInPlace, VectorOps};
pub use scoring::Scorer;
