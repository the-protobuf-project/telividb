//! The traits that define this segment's boundary.

mod vector_index;

pub use vector_index::VectorIndex;

// Re-exported so an index implementor needs one import, not two.
pub use telividb_core::ports::VectorStore;
