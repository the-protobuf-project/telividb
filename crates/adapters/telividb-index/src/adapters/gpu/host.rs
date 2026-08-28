//! Host memory, the fallback when no device reports its own.
//!
//! Needed because a CPU backend has no device memory to ask about: its
//! "device" is system RAM, and [`telividb_compute::Backend::memory`] correctly
//! returns nothing rather than inventing a figure. The budget still needs a
//! ceiling on that path, so it comes from here.

/// Total system memory, in bytes.
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
