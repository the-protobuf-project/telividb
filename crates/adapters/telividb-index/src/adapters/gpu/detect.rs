//! Asking the platform how much memory there is.
//!
//! Split from `budget.rs` because it is a different question: that file decides
//! the ceiling and enforces it, this one reports what the hardware says. The
//! readings are also the part that varies by platform, so keeping them apart
//! keeps the policy itself platform-independent.
//!
//! Everything here is a **safe** call. `candle-metal-kernels` already wraps
//! Metal's getters, so no `objc2` dependency and no `unsafe` is needed — which
//! matters because this crate is `#![forbid(unsafe_code)]`.

/// What Metal says it can use with good performance.
#[cfg(target_os = "macos")]
pub(super) fn metal_working_set_size() -> Option<usize> {
    match super::best_device() {
        candle_core::Device::Metal(metal) => {
            Some(metal.device().recommended_max_working_set_size())
        }
        _ => None,
    }
}

/// No Metal here.
#[cfg(not(target_os = "macos"))]
pub(super) fn metal_working_set_size() -> Option<usize> {
    None
}

/// Bytes Metal reports as currently allocated across the whole device.
///
/// Worth emitting beside the reservation total: the two drifting apart is how
/// you learn the registry has stopped describing reality — an allocation
/// nothing reserved, or a reservation whose memory was never freed.
#[cfg(target_os = "macos")]
pub fn device_allocated_bytes() -> Option<usize> {
    match super::best_device() {
        candle_core::Device::Metal(metal) => Some(metal.device().current_allocated_size()),
        _ => None,
    }
}

/// Only Metal reports this.
#[cfg(not(target_os = "macos"))]
pub fn device_allocated_bytes() -> Option<usize> {
    None
}

/// Total system memory, the fallback when no device figure exists.
///
/// Read through the OS rather than a crate: `libc::sysctlbyname` needs
/// `unsafe`, and this crate is `#![forbid(unsafe_code)]`. It runs once per
/// process, so a subprocess here costs nothing measurable.
pub(super) fn system_memory() -> Option<usize> {
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()?;
        String::from_utf8(out.stdout).ok()?.trim().parse().ok()
    }
    #[cfg(target_os = "linux")]
    {
        let info = std::fs::read_to_string("/proc/meminfo").ok()?;
        let line = info.lines().find(|l| l.starts_with("MemTotal:"))?;
        let kb: usize = line.split_whitespace().nth(1)?.parse().ok()?;
        Some(kb * 1024)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}
