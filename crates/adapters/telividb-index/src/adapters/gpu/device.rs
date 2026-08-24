//! Picking where the corpus lives.
//!
//! Metal, then CUDA, then CPU — the same posture invariant 7 sets for distance
//! kernels: detect at runtime, always have a correct fallback, and never
//! require a particular accelerator to *run*. A machine with no GPU answers
//! the same queries with the same results, only slower.
//!
//! Which backends are compiled in at all is decided by `candle`'s own feature
//! flags (`metal`, `cuda`), selected by whichever composition root builds this
//! crate. With neither, `new_metal`/`new_cuda` fail at the first call and this
//! resolves to CPU — so the fallback is exercised, not hypothetical.

use candle_core::Device;

/// The fastest device this build can actually reach.
///
/// Order matters and is not arbitrary: on Apple silicon, unified memory makes
/// the whole corpus GPU-addressable with no host-to-device copy, which is the
/// property that makes exhaustive search competitive (AGENT_START §14.1).
pub fn best_device() -> Device {
    if let Ok(device) = Device::new_metal(0) {
        return device;
    }
    if let Ok(device) = Device::new_cuda(0) {
        return device;
    }
    Device::Cpu
}

/// A short name for the selected device, for telemetry and for explaining a
/// query plan.
///
/// Worth emitting on every index build: a GPU path that has quietly fallen
/// back to CPU is otherwise indistinguishable from one that is working, and
/// it would pass every correctness test while delivering none of the speed.
pub fn device_name(device: &Device) -> &'static str {
    match device {
        Device::Cpu => "cpu",
        Device::Cuda(_) => "cuda",
        Device::Metal(_) => "metal",
    }
}

#[cfg(test)]
#[path = "device_test.rs"]
mod tests;
