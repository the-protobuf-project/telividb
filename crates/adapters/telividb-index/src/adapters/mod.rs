//! Implementations of [`crate::ports::VectorIndex`].

mod flat;
#[cfg(feature = "gpu")]
mod gpu;
mod hnsw;
mod memory_store;

pub use flat::FlatIndex;
#[cfg(feature = "gpu")]
pub use gpu::{
    BUDGET_ENV, BudgetSource, DEFAULT_GPU_BUDGET_FRACTION, GpuFlatIndex, best_device,
    budget_source, device_allocated_bytes, device_name, gpu_budget_bytes, gpu_resident_bytes,
};
pub use hnsw::{Graph, HnswIndex, HnswParams};
pub use memory_store::MemoryStore;
