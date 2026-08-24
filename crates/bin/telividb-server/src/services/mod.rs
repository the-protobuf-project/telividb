//! gRPC service implementations.
//!
//! Each one resolves a principal, hands work down, and maps errors back. They
//! make no access decisions of their own — that lives in the query planner so
//! the embedded path cannot bypass it.

mod collection;

pub use collection::CollectionSvc;
