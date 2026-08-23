//! The resolved collection schema.
//!
//! The engine never parses `.proto`. A `SchemaReader` adapter consumes
//! `FileDescriptorSet` bytes and resolves them into these **pure domain types**,
//! which is what keeps descriptor reflection — version-bound and I/O-shaped —
//! out of the planner, the index and the storage layer.
//!
//! Everything here is plain data: no reflection, no protobuf, no I/O.

mod collection;
mod compat;
mod vector_field;

pub use collection::{CollectionSchema, PointType};
pub use compat::{Compatibility, compare};
pub use vector_field::{IndexKind, VectorFieldSpec};
