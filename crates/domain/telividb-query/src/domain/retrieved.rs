//! What retrieval hands back, and what it takes in.
//!
//! Both carry a resource name rather than an ordinal, because an ordinal is
//! segment-local and means nothing outside the segment that produced it
//! (invariant 9). A planner that composed ordinals across a vector index and a
//! graph would be joining two different numbering schemes that happen to be
//! the same type.

use telividb_core::ResourceName;

/// One hit from a vector search, ready to seed an expansion.
#[derive(Debug, Clone, PartialEq)]
pub struct Seed {
    /// Which resource matched.
    pub name: ResourceName,
    /// Its similarity score, already in the metric's own terms.
    ///
    /// Taken as given rather than recomputed: the caller ran the search and
    /// knows whether higher or lower was nearer. See [`crate::Expansion::rank`]
    /// for how that direction is carried through.
    pub score: f32,
}

impl Seed {
    /// A seed from a search hit.
    pub fn new(name: ResourceName, score: f32) -> Self {
        Self { name, score }
    }
}

/// One result of the join, with enough provenance to be cited.
///
/// **The provenance fields are not diagnostics.** A context engine assembling
/// a prompt has to be able to say where a fragment came from — otherwise the
/// answer built from it cannot be verified, and for a memory system that is
/// the entire value. `hops` and `via` are what make a retrieved neighbour
/// explainable rather than merely present.
#[derive(Debug, Clone, PartialEq)]
pub struct Retrieved {
    /// Which resource this is.
    pub name: ResourceName,
    /// Its score after expansion decay.
    pub score: f32,
    /// Edges traversed to reach it. Zero means it was a seed — a direct
    /// similarity hit, not something the graph suggested.
    pub hops: usize,
    /// The seed this was reached from, absent when it *is* the seed.
    ///
    /// A node reachable from several seeds names the one that scored it best,
    /// which is the path a reader should be shown.
    pub via: Option<ResourceName>,
}

impl Retrieved {
    /// Whether this came from the vector search rather than the traversal.
    ///
    /// Worth distinguishing at the API boundary: a caller may reasonably want
    /// to weight direct matches differently, or to show them apart.
    pub fn is_seed(&self) -> bool {
        self.hops == 0
    }
}
