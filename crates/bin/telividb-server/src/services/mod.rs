//! gRPC service implementations.
//!
//! Each one resolves a principal, hands work down, and maps errors back. They
//! make no access decisions of their own — that lives in the query planner so
//! the embedded path cannot bypass it.

mod clock;
mod collection;
mod collection_convert;
mod embed;
pub mod models;
mod point;
mod point_batch;
mod point_convert;
mod point_create;
mod point_declare;
mod point_delete;
mod point_search;
mod point_store;
pub mod tenancy;
mod vector_search;
mod vectors;

pub use collection::CollectionSvc;
pub use embed::Embeddings;
pub use point::PointsSvc;
