//! Choosing where tensors live.

/// Which hardware a backend drives.
///
/// A closed set, so it is safe as a telemetry label — and so that a build which
/// silently fell back to the host is visible rather than merely slow, which is
/// otherwise indistinguishable from working.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    /// The host. Always available, and the correctness reference.
    Cpu,
    /// Apple GPUs, through Metal. The only accelerator on macOS.
    Metal,
    /// NVIDIA, through CUDA. Needs the CUDA toolkit at build time.
    Cuda,
    /// AMD, through HIP/ROCm. Needs ROCm at build time.
    Hip,
    /// Cross-vendor, through the Vulkan driver interface.
    Vulkan,
    /// Intel GPUs, through oneAPI.
    Sycl,
}

impl DeviceKind {
    /// The name used in configuration and telemetry.
    pub fn as_str(self) -> &'static str {
        match self {
            DeviceKind::Cpu => "cpu",
            DeviceKind::Metal => "metal",
            DeviceKind::Cuda => "cuda",
            DeviceKind::Hip => "hip",
            DeviceKind::Vulkan => "vulkan",
            DeviceKind::Sycl => "sycl",
        }
    }

    /// Whether this kind is a discrete or integrated accelerator.
    ///
    /// Used to decide whether an operation is worth dispatching at all: the
    /// host wins for small or branchy work regardless of what is installed.
    pub fn is_accelerator(self) -> bool {
        !matches!(self, DeviceKind::Cpu)
    }

    /// The kinds this build could possibly reach, best first.
    ///
    /// Compiled in rather than probed, because a backend that was not built
    /// cannot be initialised at runtime and asking would only cost time.
    pub fn compiled() -> &'static [DeviceKind] {
        &[
            #[cfg(target_os = "macos")]
            DeviceKind::Metal,
            #[cfg(feature = "cuda")]
            DeviceKind::Cuda,
            #[cfg(feature = "hip")]
            DeviceKind::Hip,
            #[cfg(feature = "vulkan")]
            DeviceKind::Vulkan,
            #[cfg(feature = "sycl")]
            DeviceKind::Sycl,
            DeviceKind::Cpu,
        ]
    }
}

/// Where a tensor lives, and the factory for obtaining one.
///
/// Callers ask for a *kind* and get whatever the build can actually provide,
/// falling back to the host. That fallback is why every operation must produce
/// identical results on every backend — only the speed may differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device {
    kind: DeviceKind,
}

impl Device {
    /// The fastest device this build can reach.
    ///
    /// The same posture the distance kernels take: detect at runtime, always
    /// have a correct fallback, and never require a particular accelerator to
    /// *run*.
    pub fn best() -> Self {
        Self {
            kind: DeviceKind::compiled()
                .first()
                .copied()
                .unwrap_or(DeviceKind::Cpu),
        }
    }

    /// The host, which is always available.
    pub fn cpu() -> Self {
        Self {
            kind: DeviceKind::Cpu,
        }
    }

    /// A device of a specific kind, if this build has one.
    pub fn of(kind: DeviceKind) -> crate::Result<Self> {
        match DeviceKind::compiled().contains(&kind) {
            true => Ok(Self { kind }),
            false => Err(crate::Error::BackendUnavailable {
                kind: kind.as_str(),
            }),
        }
    }

    /// Which hardware this is.
    pub fn kind(self) -> DeviceKind {
        self.kind
    }

    /// Whether work dispatched here leaves the host.
    pub fn is_accelerator(self) -> bool {
        self.kind.is_accelerator()
    }
}

impl Default for Device {
    /// The fastest available, matching [`Device::best`].
    fn default() -> Self {
        Self::best()
    }
}

#[cfg(test)]
#[path = "device_test.rs"]
mod tests;
