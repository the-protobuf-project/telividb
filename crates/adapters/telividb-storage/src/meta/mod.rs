//! Small, mutable, `redb`-backed metadata — today, the graph's edges and the
//! document service's points.

mod factory;
mod point_read;
mod point_record;
mod redb_graph_store;
mod redb_point_store;
mod row_binding;

pub use factory::{GraphStoreConfig, PointStoreConfig, open_graph_store, open_point_store};
pub use redb_graph_store::RedbGraphStore;
pub use redb_point_store::RedbPointStore;
