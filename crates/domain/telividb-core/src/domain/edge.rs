//! A typed relationship between two points.
//!
//! Nodes are points that already exist — they have ids and payloads. Edges are
//! new: a directed, typed relationship one point declares toward another. See
//! ARCHITECTURE.md §3 and CLAUDE.md rule 47.

use super::ResourceName;

/// A directed edge from one resource to another.
///
/// Endpoints are [`ResourceName`]s rather than internal ordinals, for the same
/// reason external ids are the only portable identity everywhere else in this
/// crate (CLAUDE.md invariant 9): an edge that leaked an ordinal would be
/// meaningless the moment it crossed a segment boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// The resource the edge points from.
    pub src: ResourceName,
    /// The resource the edge points to.
    pub dst: ResourceName,
    /// What kind of relationship this is, e.g. `"HAS_SHOT"` or `"MENTIONS"`.
    ///
    /// A string rather than an enum: edge types are declared per collection
    /// schema, not fixed by the engine — the same reason point types are not
    /// a closed set.
    pub edge_type: String,
    /// Strength or salience of the relationship, for ranking traversal
    /// results. `1.0` when the relationship has no natural weight.
    pub weight: f32,
}

impl Edge {
    /// Build an edge with the given endpoints, type and weight.
    pub fn new(
        src: ResourceName,
        dst: ResourceName,
        edge_type: impl Into<String>,
        weight: f32,
    ) -> Self {
        Self {
            src,
            dst,
            edge_type: edge_type.into(),
            weight,
        }
    }
}

#[cfg(test)]
#[path = "edge_test.rs"]
mod tests;
