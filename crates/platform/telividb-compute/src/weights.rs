//! A GGUF file, resident on a device.
//!
//! Loading is two contexts and one copy:
//!
//! 1. `gguf_init_from_file` with a non-null `ctx` parses the header and gives
//!    back a **host** context holding every tensor with its data read from the
//!    file.
//! 2. A second, `no_alloc` context declares the same tensors again, and
//!    `ggml_backend_alloc_ctx_tensors` gives them **device** storage.
//! 3. Each tensor's bytes are written across, then the host context is freed.
//!
//! The host copy is released as soon as the device has the bytes, so a resident
//! model costs its device footprint and not twice that.
//!
//! **Quantized tensors are moved verbatim.** A `Q4_K` weight stays `Q4_K` on
//! the device, and [`Tensor::matmul`] multiplies it against f32 activations
//! natively. Nothing dequantizes, which is the whole reason a GGUF model runs at
//! its stated memory footprint rather than at f32 size — and it is what a
//! runtime with no quantized matmul cannot do without unpacking every weight.
//!
//! [`Tensor::matmul`]: crate::Tensor::matmul

use crate::backend::Backend;
use crate::error::{Error, Result};
use crate::sys;
use std::collections::HashMap;
use std::ffi::CString;

/// Every tensor of one GGUF, resident on a backend.
pub struct Weights {
    /// Metadata for the device-resident tensors.
    ctx: *mut sys::ggml_context,
    /// The device allocation backing all of them.
    buffer: sys::ggml_backend_buffer_t,
    /// The parsed header, kept for metadata lookups.
    gguf: *mut sys::gguf_context,
    /// Tensor by name, so a model finds `blk.0.attn_q.weight` without a scan.
    by_name: HashMap<String, *mut sys::ggml_tensor>,
}

// SAFETY: the pointers are owned by this value and freed exactly once in
// `Drop`. `Weights` is `Send` and deliberately not `Sync` — the backend it
// lives on holds one command queue, so concurrent use would race (rule 46).
unsafe impl Send for Weights {}

impl Weights {
    /// Read `path` and copy every tensor onto `backend`.
    pub fn load(path: &std::path::Path, backend: &Backend) -> Result<Self> {
        let c_path =
            CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_| Error::Runtime {
                op: "gguf_open",
                reason: "the model path contains an interior NUL byte".to_owned(),
            })?;

        let mut host_ctx: *mut sys::ggml_context = std::ptr::null_mut();
        // SAFETY: `c_path` is NUL-terminated and outlives the call. `no_alloc:
        // false` with a non-null `ctx` is what makes ggml read tensor *data*
        // rather than only the header.
        let gguf = unsafe {
            sys::gguf_init_from_file(
                c_path.as_ptr(),
                sys::gguf_init_params {
                    no_alloc: false,
                    ctx: &mut host_ctx,
                },
            )
        };
        if gguf.is_null() || host_ctx.is_null() {
            return Err(Error::Runtime {
                op: "gguf_open",
                reason: format!("{} is not a readable GGUF file", path.display()),
            });
        }

        match Self::upload(host_ctx, gguf, backend) {
            Ok((ctx, buffer, by_name)) => {
                // The bytes are on the device now; the host copy has no further
                // use and holds the whole model.
                // SAFETY: `host_ctx` is live and freed exactly once here.
                unsafe { sys::ggml_free(host_ctx) };
                Ok(Self {
                    ctx,
                    buffer,
                    gguf,
                    by_name,
                })
            }
            Err(e) => {
                // SAFETY: both are live and unfreed on this path.
                unsafe {
                    sys::ggml_free(host_ctx);
                    sys::gguf_free(gguf);
                }
                Err(e)
            }
        }
    }

    /// How many tensors the file carried.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Whether the file carried no tensors at all.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// The parsed header, for the metadata accessors.
    pub(crate) fn gguf(&self) -> *mut sys::gguf_context {
        self.gguf
    }

    /// The device tensor named `name`, for operations in this crate.
    pub(crate) fn raw_tensor(&self, name: &str) -> Option<*mut sys::ggml_tensor> {
        self.by_name.get(name).copied()
    }
}

impl Drop for Weights {
    fn drop(&mut self) {
        // SAFETY: each was produced in `load` and is freed exactly once. The
        // buffer goes before the context describing the tensors inside it.
        unsafe {
            sys::ggml_backend_buffer_free(self.buffer);
            sys::ggml_free(self.ctx);
            sys::gguf_free(self.gguf);
        }
    }
}

#[cfg(test)]
#[path = "weights_test.rs"]
mod tests;
