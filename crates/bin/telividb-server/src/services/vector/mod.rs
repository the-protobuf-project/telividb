//! Vectors: the named fields a point carries, and turning text into them.
//!
//! Grouped with the embedding path rather than under `point/` because these are
//! about the *vector* side of a point — which model produced it, how wide it
//! is, how a query encodes — and that is the concern rules 12, 17 and 18 keep
//! separate from the row itself.

pub(crate) mod convert;
mod embed;
mod fields;
mod resident;
mod search;

pub(crate) use fields::VectorFields;
pub use resident::Embeddings;
