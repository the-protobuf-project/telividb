//! A live ggml backend, and the graph runs dispatched to it.
//!
//! ggml is **graph-based rather than eager**: tensors are declared into a
//! context, wired into a graph, and computed in one call. That shapes the API
//! above it — there is no free-floating `a.matmul(b)` that returns a value,
//! because a lone operation still costs a context, a graph and a dispatch.
//! Operations are therefore expressed as whole jobs, which is also the shape
//! that keeps a device busy.

use crate::device::{Device, DeviceKind};
use crate::error::{Error, Result};
use crate::sys;

/// What a device reports about its own memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Memory {
    /// Bytes currently available.
    pub free: usize,
    /// Bytes the device has in total.
    pub total: usize,
}

impl Memory {
    /// Bytes currently in use, as the device sees it.
    pub fn used(self) -> usize {
        self.total.saturating_sub(self.free)
    }
}

/// An initialised compute backend.
///
/// Owns the underlying `ggml_backend_t` and frees it on drop, which is why no
/// caller ever holds the raw handle.
pub struct Backend {
    raw: sys::ggml_backend_t,
    device: Device,
}

// SAFETY: a ggml backend is not tied to the thread that created it, and this
// wrapper hands out no interior handle. Every method takes `&mut self` or is
// read-only, so two threads cannot drive one backend concurrently.
unsafe impl Send for Backend {}

impl Backend {
    /// Initialise the fastest backend this build can reach.
    ///
    /// Falls back to the host rather than failing: a machine without a GPU must
    /// still answer queries, and every backend produces identical results.
    ///
    /// The preferred kind comes from [`Device::best`], which is what makes
    /// `TELIVIDB_DEVICE` mean the same thing everywhere. Deciding it a second
    /// time here is how the two came to disagree: the announcement at startup
    /// reported the overridden device while the index quietly initialised
    /// another, and nothing surfaced the contradiction.
    pub fn best() -> Result<Self> {
        let preferred = Device::best().kind();
        if let Ok(backend) = Self::of(preferred) {
            return Ok(backend);
        }
        for kind in DeviceKind::compiled() {
            if let Ok(backend) = Self::of(*kind) {
                return Ok(backend);
            }
        }
        Self::of(DeviceKind::Cpu)
    }

    /// Initialise a backend of a specific kind.
    pub fn of(kind: DeviceKind) -> Result<Self> {
        let device = Device::of(kind)?;

        // SAFETY: `ggml_backend_init_by_type` takes a type tag and an optional
        // parameter string; null selects the default device for that type. It
        // returns null when the backend is not present, which is checked below.
        let raw = unsafe {
            let tag = match kind {
                DeviceKind::Cpu => sys::ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_CPU,
                _ => sys::ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_GPU,
            };
            sys::ggml_backend_init_by_type(tag, std::ptr::null_mut())
        };

        if raw.is_null() {
            return Err(Error::BackendUnavailable {
                kind: kind.as_str(),
            });
        }
        Ok(Self { raw, device })
    }

    /// Free and total device memory, in bytes.
    ///
    /// `None` where the backend reports nothing, which is the honest answer for
    /// a host backend whose "device memory" is just system RAM.
    ///
    /// Worth having per backend rather than per platform: this reports for CUDA,
    /// HIP, Vulkan and Metal alike, where the previous Metal-only reading left
    /// every other accelerator budgeting against a guess.
    pub fn memory(&self) -> Option<Memory> {
        if !self.device.is_accelerator() {
            return None;
        }

        // SAFETY: `raw` is non-null by construction; `ggml_backend_get_device`
        // returns the device that backend was opened on, and `dev_memory`
        // writes two `usize` out-params it is given valid pointers for.
        let (free, total) = unsafe {
            let dev = sys::ggml_backend_get_device(self.raw);
            if dev.is_null() {
                return None;
            }
            let mut free = 0usize;
            let mut total = 0usize;
            sys::ggml_backend_dev_memory(dev, &mut free, &mut total);
            (free, total)
        };

        // A backend that reports nothing returns zeroes rather than failing,
        // and a zero total would make every budget fraction zero — which reads
        // as "no memory" rather than "no answer".
        match total {
            0 => None,
            _ => Some(Memory { free, total }),
        }
    }

    /// The raw handle, for the operations in this crate only.
    pub(crate) fn raw(&self) -> sys::ggml_backend_t {
        self.raw
    }

    /// Which hardware this backend drives.
    pub fn device(&self) -> Device {
        self.device
    }

    /// The backend's own name, as ggml reports it.
    ///
    /// Worth surfacing beside [`Self::device`]: the kind says what was asked
    /// for, this says what was actually opened.
    pub fn name(&self) -> String {
        // SAFETY: `raw` is non-null by construction and ggml returns a
        // NUL-terminated static string that outlives this borrow.
        unsafe {
            let ptr = sys::ggml_backend_name(self.raw);
            match ptr.is_null() {
                true => String::new(),
                false => std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned(),
            }
        }
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        // SAFETY: `raw` was produced by `ggml_backend_init_by_type` and is
        // freed exactly once, because `Backend` is not `Clone`.
        unsafe { sys::ggml_backend_free(self.raw) }
    }
}

#[cfg(test)]
#[path = "backend_test.rs"]
mod tests;
