//! Getting host data into a graph.
//!
//! Split from `context.rs` because that file is about a graph's lifetime — the
//! arena, the allocator, the compute call — and this is about the one thing
//! that crosses from host memory into it. The distinction matters on a device:
//! an input is the only place a copy happens, so it is the only place worth
//! looking when a transfer shows up in a profile.

use crate::context::Context;
use crate::error::{Error, Result};
use crate::sys;
use crate::tensor::Tensor;

impl<'b> Context<'b> {
    /// An input tensor of `f32`, filled from `data`.
    ///
    /// `shape` is in ggml order — **fastest-varying dimension first**, the
    /// reverse of the row-major convention most array libraries print. A
    /// `(dim, tokens)` tensor holds one vector per column, contiguous.
    pub fn input_f32(&self, data: &[f32], shape: [usize; 2]) -> Result<Tensor<'_, 'b>> {
        self.input(data, shape, sys::ggml_type_GGML_TYPE_F32)
    }

    /// An input tensor of `i32` — token ids, positions, and anything else
    /// consumed by [`Tensor::rows`], which indexes rather than computes.
    pub fn input_i32(&self, data: &[i32], shape: [usize; 2]) -> Result<Tensor<'_, 'b>> {
        self.input(data, shape, sys::ggml_type_GGML_TYPE_I32)
    }

    /// Create a two-dimensional input, give it storage, and write `data` in.
    fn input<T>(
        &self,
        data: &[T],
        shape: [usize; 2],
        kind: sys::ggml_type,
    ) -> Result<Tensor<'_, 'b>> {
        let [ne0, ne1] = shape;
        if data.len() != ne0 * ne1 {
            return Err(Error::ShapeMismatch {
                expected: format!("{} elements", ne0 * ne1),
                actual: format!("{}", data.len()),
            });
        }

        // SAFETY: the context is live with metadata room reserved in `new`, and
        // both extents are checked non-zero below via the null result.
        let raw = unsafe { sys::ggml_new_tensor_2d(self.raw, kind, ne0 as i64, ne1 as i64) };
        if raw.is_null() {
            return Err(Error::Allocation {
                bytes: std::mem::size_of_val(data),
            });
        }

        // Allocates only tensors that do not already have storage, so earlier
        // inputs keep the buffers they were given. See the module note.
        //
        // SAFETY: `raw` is a live context and `backend` a live backend.
        let buffer = unsafe { sys::ggml_backend_alloc_ctx_tensors(self.raw, self.backend.raw()) };
        if buffer.is_null() {
            return Err(Error::Allocation {
                bytes: std::mem::size_of_val(data),
            });
        }
        self.buffers.borrow_mut().push(buffer);

        // SAFETY: `raw` was just given storage of exactly this size.
        unsafe {
            sys::ggml_backend_tensor_set(raw, data.as_ptr().cast(), 0, std::mem::size_of_val(data))
        };
        Ok(Tensor::new(self, raw))
    }
}
