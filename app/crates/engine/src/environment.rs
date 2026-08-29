//! What this machine can actually do, detected rather than configured.
//!
//! The one fact no orchestrator can see from outside the process: a build that
//! quietly fell back to the CPU passes every correctness test while delivering
//! none of the speed. From outside the pod is healthy, the GPU is allocated,
//! and nothing is using it.
//!
//! So the window reports the backend that was *selected*, not the one that was
//! compiled in — those differ, and the difference is the whole point.

use serde::Serialize;
use telividb_compute::{Backend, Device};

/// The compute environment, as this process found it.
#[derive(Debug, Clone, Serialize)]
pub struct Environment {
    /// The selected backend: `metal`, `cuda`, `cpu`, and so on.
    pub backend: String,

    /// Device memory in bytes, when the backend reports it.
    ///
    /// `None` rather than zero when unknown. A host backend has no separate
    /// device memory to report, and a virtualised GPU may decline to say —
    /// both are "not applicable", which is different from "no memory".
    pub total_bytes: Option<u64>,

    /// Free device memory in bytes, when the backend reports it.
    pub free_bytes: Option<u64>,

    /// Whether the selection came from `TELIVIDB_DEVICE` rather than detection.
    ///
    /// Worth showing: an operator who pinned the host and then wondered why it
    /// is slow should be able to see that they did.
    pub overridden: bool,
}

impl Environment {
    /// Detect the environment this process is running in.
    ///
    /// Initialising a backend is what makes the memory readable, and it is the
    /// same work the index does on its first search — so this pays it once, at
    /// startup, where a window can show the answer instead of a spinner.
    pub fn detect() -> Self {
        let device = Device::best();
        let memory = Backend::of(device.kind()).ok().and_then(|b| b.memory());

        Self {
            backend: device.kind().as_str().to_owned(),
            total_bytes: memory.map(|m| m.total as u64),
            free_bytes: memory.map(|m| m.free as u64),
            overridden: std::env::var_os("TELIVIDB_DEVICE").is_some(),
        }
    }
}
