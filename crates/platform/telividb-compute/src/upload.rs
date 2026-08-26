//! Moving a parsed GGUF's tensors from host memory onto a device.
//!
//! Split from `weights.rs` so that file stays about *what a loaded model is*
//! while this is about the copy that makes it resident — the step where a
//! mistake costs either correctness (a tensor left unwritten) or twice the
//! memory (the host copy outliving the upload).

use crate::backend::Backend;
use crate::error::{Error, Result};
use crate::sys;
use crate::weights::Weights;
use std::collections::HashMap;
use std::ffi::CStr;

impl Weights {
    /// Declare every host tensor again on the device and copy the bytes across.
    #[allow(clippy::type_complexity)]
    pub(crate) fn upload(
        host_ctx: *mut sys::ggml_context,
        gguf: *mut sys::gguf_context,
        backend: &Backend,
    ) -> Result<(
        *mut sys::ggml_context,
        sys::ggml_backend_buffer_t,
        HashMap<String, *mut sys::ggml_tensor>,
    )> {
        // SAFETY: `gguf` is a live parsed header.
        let count = unsafe { sys::gguf_get_n_tensors(gguf) }.max(0) as usize;

        // SAFETY: plain-data params; the result is null-checked below.
        let ctx = unsafe {
            sys::ggml_init(sys::ggml_init_params {
                mem_size: sys::ggml_tensor_overhead().saturating_mul(count + 1),
                mem_buffer: std::ptr::null_mut(),
                no_alloc: true,
            })
        };
        if ctx.is_null() {
            return Err(Error::Allocation { bytes: 0 });
        }

        // SAFETY: every pointer below comes from a live context. `dup_tensor`
        // copies shape and type only — `no_alloc` means no storage yet.
        let mut by_name = HashMap::with_capacity(count);
        unsafe {
            let mut src = sys::ggml_get_first_tensor(host_ctx);
            while !src.is_null() {
                let copy = sys::ggml_dup_tensor(ctx, src);
                if copy.is_null() {
                    sys::ggml_free(ctx);
                    return Err(Error::Allocation { bytes: 0 });
                }
                sys::ggml_set_name(copy, sys::ggml_get_name(src));
                let name = CStr::from_ptr(sys::ggml_get_name(src))
                    .to_string_lossy()
                    .into_owned();
                by_name.insert(name, copy);
                src = sys::ggml_get_next_tensor(host_ctx, src);
            }

            let buffer = sys::ggml_backend_alloc_ctx_tensors(ctx, backend.raw());
            if buffer.is_null() {
                sys::ggml_free(ctx);
                return Err(Error::Allocation { bytes: 0 });
            }

            // Now that the device tensors have storage, copy each one's bytes.
            // `nbytes` is the *quantized* size where the tensor is quantized,
            // so this moves the compact form rather than expanding it.
            let mut src = sys::ggml_get_first_tensor(host_ctx);
            while !src.is_null() {
                let name = CStr::from_ptr(sys::ggml_get_name(src)).to_string_lossy();
                if let Some(&dst) = by_name.get(name.as_ref()) {
                    sys::ggml_backend_tensor_set(dst, (*src).data, 0, sys::ggml_nbytes(src));
                }
                src = sys::ggml_get_next_tensor(host_ctx, src);
            }

            Ok((ctx, buffer, by_name))
        }
    }
}
