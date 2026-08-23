//! The coarse tier of a two-tier field.
//!
//! [`VectorStore`](super::VectorStore) hands back `&[f32]`, which a quantized
//! store cannot do without materializing a row. Rather than widen that trait —
//! which would force every implementor to care about codecs — a field exposes
//! its **two tiers separately** and the search path composes them:
//!
//! ```text
//! ScanTier      wide, cheap, approximate   → candidates
//! VectorStore   narrow, exact              → rerank the candidates
//! ```
//!
//! The split is what keeps flexibility. A field may have no scan tier at all,
//! in which case search runs exactly against `VectorStore` and nothing else
//! changes; or it may gain PQ later without the index learning anything new.
//! Deciding which tiers to use is the planner's job, not the store's.

use crate::{Metric, Ordinal, Result};

/// A compressed representation that can be scored without full precision.
///
/// Implementors score directly against their codes where the codec allows it —
/// Hamming distance over binary, or an asymmetric distance table for PQ — so a
/// scan need not decode every row it rejects. That is most of where the speed
/// comes from.
pub trait ScanTier: Send + Sync {
    /// Codec name, for telemetry and for explaining a query plan.
    fn codec(&self) -> &'static str;

    /// Rows covered, including absent ones.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Prepare a query for repeated scoring against this tier.
    ///
    /// Encoding a query once and reusing it is the difference between a scan
    /// that costs one transform and one that costs a transform per row. For PQ
    /// this is where the distance table is built.
    fn prepare(&self, query: &[f32], metric: Metric) -> Result<PreparedQuery>;

    /// Score one row against a prepared query.
    ///
    /// Returns `None` for a row with no value for this field — normal in a
    /// multimodal collection, and it must not be scored as though it were zeros.
    /// The scale is the metric's own, so results are directly comparable with
    /// exact scores; only the precision differs.
    fn score(&self, prepared: &PreparedQuery, ordinal: Ordinal) -> Option<f32>;
}

/// A query transformed into whatever form a scan tier scores against.
///
/// Deliberately opaque and owned: a tier may need the query re-encoded, a
/// lookup table, or nothing at all, and the search path should not branch on
/// which.
#[derive(Debug, Clone)]
pub struct PreparedQuery {
    /// The metric this was prepared for; scores are on its scale.
    pub metric: Metric,
    /// Tier-specific state — codes, a distance table, or the raw query.
    pub state: PreparedState,
}

/// What a tier needs to score a row.
#[derive(Debug, Clone)]
pub enum PreparedState {
    /// The query itself, for tiers that decode before scoring.
    Vector(Vec<f32>),
    /// The query encoded into the tier's own code space.
    Codes(Vec<u8>),
    /// Precomputed distances from the query to every centroid, per subspace.
    ///
    /// The asymmetric form: the query stays full precision and only stored
    /// vectors are quantized, which is more accurate than encoding both.
    Table {
        subspaces: usize,
        distances: Vec<f32>,
    },
}

impl PreparedQuery {
    pub fn vector(metric: Metric, query: Vec<f32>) -> Self {
        Self {
            metric,
            state: PreparedState::Vector(query),
        }
    }

    pub fn codes(metric: Metric, codes: Vec<u8>) -> Self {
        Self {
            metric,
            state: PreparedState::Codes(codes),
        }
    }

    pub fn table(metric: Metric, subspaces: usize, distances: Vec<f32>) -> Self {
        Self {
            metric,
            state: PreparedState::Table {
                subspaces,
                distances,
            },
        }
    }
}
