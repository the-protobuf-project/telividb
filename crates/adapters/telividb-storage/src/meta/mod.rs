//! Small, mutable metadata: the collection catalogue, point records, the
//! graph's edges, and the tenancy tree.
//!
//! One directory per backend, and beneath it one per resource. The store, the
//! record codec and their tests sit together, so the question "how is a point
//! stored?" is answered by one folder rather than by five files that share a
//! prefix.

mod factory;
mod redb;

pub use factory::{GraphStoreConfig, PointStoreConfig, open_graph_store, open_point_store};
pub use redb::{RedbCollectionStore, RedbGraphStore, RedbPointStore, RedbTenancyStore};
