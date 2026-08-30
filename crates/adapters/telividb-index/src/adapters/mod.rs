//! Implementations of [`crate::ports::VectorIndex`].

mod flat;
#[cfg(feature = "gpu")]
mod gpu;
mod hnsw;
mod ivf;
mod memory_store;

pub use flat::FlatIndex;
#[cfg(feature = "gpu")]
pub use gpu::{
    BUDGET_ENV, BudgetSource, DEFAULT_GPU_BUDGET_FRACTION, GpuFlatIndex, budget_source,
    device_allocated_bytes, device_name, gpu_budget_bytes, gpu_resident_bytes,
};
pub use hnsw::{Graph, HnswIndex, HnswParams};
pub use ivf::{Coarse, IvfFlatIndex, IvfParams, IvfPqIndex};
pub use memory_store::MemoryStore;
/// Where a device-resident index can run, re-exported so a caller choosing one
/// needs no direct dependency on the runtime crate.
#[cfg(feature = "gpu")]
pub use telividb_compute::{Device, DeviceKind};
