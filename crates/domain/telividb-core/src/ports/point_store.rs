//! What the document service is allowed to see of storage.
//!
//! Lives in `core` for the same reason [`GraphStore`](crate::ports::GraphStore)
//! does: `telividb-storage` implements it, and callers above storage consume
//! it, so the boundary belongs to neither on its own.

use crate::{Point, ResourceName, Result};

/// Read access to the points of one collection.
///
/// Deliberately read-only, mirroring `GraphStore` and `VectorStore`: creating
/// and deleting points are concerns of the concrete adapter (e.g.
/// `RedbPointStore::create`), not of this port.
pub trait PointStore: Send + Sync {
    /// The point named `name`, or `None` if it does not exist.
    fn get(&self, name: &ResourceName) -> Result<Option<Point>>;

    /// Every point whose resource name is a direct child of `parent`.
    ///
    /// `parent` is a collection resource name, e.g. `collections/media` —
    /// the same value `ListPointsRequest.parent` carries.
    fn list(&self, parent: &ResourceName) -> Result<Vec<Point>>;
}
