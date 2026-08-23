//! Implementations of [`crate::ports::VectorIndex`].

mod flat;
mod memory_store;

pub use flat::FlatIndex;
pub use memory_store::MemoryStore;
