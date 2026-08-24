//! What the graph layer is allowed to see of storage.
//!
//! This lives in `core` rather than in either neighbour because both
//! implement or consume it: `telividb-storage` implements it against `redb`,
//! and `telividb-graph` consumes it to rehydrate an in-memory graph — the same
//! reason [`VectorStore`](crate::ports::VectorStore) lives here rather than in
//! either of *its* neighbours.

use crate::{Edge, Result};

/// Read access to every edge of one collection.
///
/// Deliberately read-only, mirroring `VectorStore`: mutation is a concern of
/// the concrete adapter (e.g. `RedbGraphStore::insert_edge`), not of this
/// port. A caller that only ever reads a store — which is every consumer
/// except the write path itself — should not be able to mutate one it was
/// handed.
pub trait GraphStore: Send + Sync {
    /// Every edge this store holds, in no particular order.
    ///
    /// The graph layer rehydrates a whole collection's edges into memory on
    /// load (CLAUDE.md rule 47) rather than fetching them node by node, so
    /// this is a full scan, not a point lookup. Each item is independently
    /// fallible because a decode failure partway through a scan (a corrupt
    /// key, say) must surface as an error on that one edge, not abort the
    /// iterator silently or panic.
    fn iter_edges(&self) -> Result<Box<dyn Iterator<Item = Result<Edge>> + '_>>;
}
