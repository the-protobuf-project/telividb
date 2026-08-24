//! gRPC service implementations.
//!
//! Each one resolves a principal, hands work down, and maps errors back. They
//! make no access decisions of their own — that lives in the query planner so
//! the embedded path cannot bypass it.

mod collection;
mod embed;
mod point;
mod point_batch;
mod point_convert;
mod point_create;
mod point_search;
mod point_store;
mod vector_search;
mod vectors;

pub use collection::CollectionSvc;
pub use embed::Embeddings;
pub use point::PointsSvc;
