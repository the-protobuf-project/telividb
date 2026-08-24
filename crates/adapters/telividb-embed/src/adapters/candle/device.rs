//! Picking where a model runs.
//!
//! Deliberately the same order and the same posture as the GPU index's own
//! selection: Metal, then CUDA, then CPU, with a correct fallback always
//! present (invariant 7). Duplicated rather than shared because the two crates
//! do not depend on each other and `telividb-index`'s copy is behind its `gpu`
//! feature — a shared helper would mean one crate reaching into the other's
//! optional feature, which is the outward dependency rule 14 forbids.

use candle_core::Device;

/// The fastest device this build can actually reach.
pub fn best_device() -> Device {
    if let Ok(device) = Device::new_metal(0) {
        return device;
    }
    if let Ok(device) = Device::new_cuda(0) {
        return device;
    }
    Device::Cpu
}

/// A short name for the selected device, for telemetry.
///
/// Worth emitting on every load: a model that has quietly fallen back to CPU
/// is otherwise indistinguishable from one on the GPU — it returns identical
/// vectors, just far more slowly.
pub fn device_name(device: &Device) -> &'static str {
    match device {
        Device::Cpu => "cpu",
        Device::Cuda(_) => "cuda",
        Device::Metal(_) => "metal",
    }
}
