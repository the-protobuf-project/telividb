//! Distance kernels.
//!
//! Rudimentary for now: scalar only. The dispatch seam exists so AVX2 / AVX-512
//! / NEON implementations slot in behind it without touching callers, and so
//! the scalar path stays as the always-correct reference the SIMD versions are
//! tested against. See CLAUDE.md invariant 7.
#![forbid(unsafe_code)]

mod scalar;

pub use scalar::{dot, l2_squared, normalize};

use episteme_core::Metric;

/// Score `query` against `candidate` under `metric`.
///
/// Ordering follows [`Metric::higher_is_nearer`] — callers must not assume that
/// a larger score always means a nearer neighbour.
///
/// # Panics
/// Panics if the slices differ in length; callers validate dimension before the
/// hot path rather than paying for a check per comparison.
pub fn score(metric: Metric, query: &[f32], candidate: &[f32]) -> f32 {
    debug_assert_eq!(query.len(), candidate.len(), "dimension checked by caller");
    match metric {
        // Cosine is normalised at ingest, so it *is* dot by this point.
        Metric::Dot | Metric::Cosine => dot(query, candidate),
        Metric::L2 => l2_squared(query, candidate),
    }
}
