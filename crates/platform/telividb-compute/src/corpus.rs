//! A corpus of vectors, resident on a device.
//!
//! Uploaded once and scored many times — that ratio is the whole reason a
//! device is worth using here. A corpus that had to be re-uploaded per query
//! would spend more time copying than scoring, which is why this owns its
//! device memory rather than borrowing a caller's slice.
//!
//! # Layout
//!
//! ggml's fastest-varying dimension is `ne[0]`, so a corpus is created as
//! `(dim, rows)` and its bytes are `[vec0.., vec1.., ...]` — exactly the layout
//! a `Vec<f32>` of row-major vectors already has. No transpose on upload.

use crate::backend::Backend;
use crate::error::{Error, Result};
use crate::sys;

/// Vectors held in device memory, ready to score against.
pub struct Corpus {
    backend: Backend,
    /// Holds tensor *metadata*; the bytes live in `buffer`.
    ctx: *mut sys::ggml_context,
    buffer: sys::ggml_backend_buffer_t,
    tensor: *mut sys::ggml_tensor,
    rows: usize,
    dim: usize,
}

// SAFETY: the context, buffer and tensor are owned by this value and freed
// exactly once in `Drop`. No handle escapes, and every method takes `&self`
// while performing no interior mutation of the corpus itself.
unsafe impl Send for Corpus {}

impl Corpus {
    /// Copy `vectors` onto the device behind `backend`.
    ///
    /// `vectors` is `rows * dim` floats, each vector contiguous. The caller
    /// therefore holds the whole corpus in host memory for the duration; see
    /// [`Corpus::staged`] where that second copy is the expensive part.
    pub fn upload(backend: Backend, vectors: &[f32], rows: usize, dim: usize) -> Result<Self> {
        if vectors.len() != rows * dim {
            return Err(Error::ShapeMismatch {
                expected: format!("{} floats", rows * dim),
                actual: format!("{}", vectors.len()),
            });
        }

        let mut staged = Self::staged(backend, rows, dim)?;
        staged.push_rows(vectors)?;
        staged.finish()
    }

    /// Device memory for `rows * dim` floats, with nothing written to it yet.
    pub(crate) fn empty(backend: Backend, rows: usize, dim: usize) -> Result<Self> {
        if rows == 0 || dim == 0 {
            return Err(Error::ShapeMismatch {
                expected: "a non-empty corpus".to_owned(),
                actual: format!("{rows} x {dim}"),
            });
        }

        // Metadata only — `no_alloc` leaves the bytes to the backend allocator
        // below, which is what puts them in device memory rather than here.
        // SAFETY: the params are plain data and the returned context is
        // checked for null before use.
        let ctx = unsafe {
            let overhead = sys::ggml_tensor_overhead().saturating_mul(2);
            sys::ggml_init(sys::ggml_init_params {
                mem_size: overhead,
                mem_buffer: std::ptr::null_mut(),
                no_alloc: true,
            })
        };
        if ctx.is_null() {
            return Err(Error::Allocation { bytes: 0 });
        }

        // From here every early return must free `ctx`, so the work is done in
        // a closure and the context released once at the end.
        let built = Self::allocate(&backend, ctx, rows, dim);
        match built {
            Ok((buffer, tensor)) => Ok(Self {
                backend,
                ctx,
                buffer,
                tensor,
                rows,
                dim,
            }),
            Err(e) => {
                // SAFETY: `ctx` is non-null and has not been freed on this path.
                unsafe { sys::ggml_free(ctx) };
                Err(e)
            }
        }
    }

    /// Create the tensor, back it with device memory, and fill it.
    fn allocate(
        backend: &Backend,
        ctx: *mut sys::ggml_context,
        rows: usize,
        dim: usize,
    ) -> Result<(sys::ggml_backend_buffer_t, *mut sys::ggml_tensor)> {
        // SAFETY: `ctx` is a live context with room for this tensor's metadata,
        // and the dimensions are non-zero and fit `i64` by the checks above.
        let tensor = unsafe {
            sys::ggml_new_tensor_2d(ctx, sys::ggml_type_GGML_TYPE_F32, dim as i64, rows as i64)
        };
        if tensor.is_null() {
            return Err(Error::Allocation {
                bytes: rows * dim * 4,
            });
        }

        // SAFETY: allocates device memory for every tensor in `ctx` and returns
        // null if the device cannot satisfy it — which is checked, because an
        // over-large upload otherwise aborts rather than failing.
        let buffer = unsafe { sys::ggml_backend_alloc_ctx_tensors(ctx, backend.raw()) };
        if buffer.is_null() {
            return Err(Error::Allocation {
                bytes: rows * dim * 4,
            });
        }

        Ok((buffer, tensor))
    }

    /// How many vectors are resident.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// The width every vector shares.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Bytes this corpus occupies on the device.
    pub fn bytes(&self) -> usize {
        self.rows * self.dim * std::mem::size_of::<f32>()
    }

    /// The backend this corpus lives on.
    pub fn backend(&self) -> &Backend {
        &self.backend
    }

    /// The device-side tensor, for operations in this crate only.
    pub(crate) fn tensor(&self) -> *mut sys::ggml_tensor {
        self.tensor
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        // SAFETY: both were produced by this value's constructor and are freed
        // exactly once, because `Corpus` is not `Clone`. The buffer is released
        // before the context that describes it.
        unsafe {
            sys::ggml_backend_buffer_free(self.buffer);
            sys::ggml_free(self.ctx);
        }
    }
}
