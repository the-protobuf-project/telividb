//! gRPC service implementations.
//!
//! Each one resolves a principal, hands work down, and maps errors back. They
//! make no access decisions of their own — that lives in the query planner so
//! the embedded path cannot bypass it.
//!
//! One directory per service, because a service is several files: the `tonic`
//! trait impl, the conversions, and a file per operation that has enough rules
//! to be worth reading on its own. Flat, the prefixes did the grouping — a
//! dozen `point_*.rs` in one listing — and a reader had to know the convention
//! before the shape was visible.

mod clock;
pub mod collection;
pub mod models;
pub mod point;
pub mod system;
pub mod tenancy;
pub mod vector;

pub use collection::CollectionSvc;
pub use point::PointsSvc;
pub use vector::Embeddings;
