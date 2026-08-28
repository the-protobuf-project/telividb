//! A graph under construction, and the memory it needs.
//!
//! **ggml is graph-based, not eager.** Declaring `a.matmul(b)` computes
//! nothing: it records a node and hands back a handle. Work happens once, in
//! [`Context::compute`], for the whole graph. That is the shape rule 42
//! describes — the API expresses *whole jobs* — and it is also the only shape
//! that keeps a device busy, so the constraint and the performance advice point
//! the same way.
//!
//! # Two kinds of tensor, and why allocation is split
//!
//! An **input** carries data from the host and must have storage before it can
//! be written to. An **intermediate** is produced by an operation and only needs
//! storage while the graph runs. ggml allocates them differently:
//! `ggml_backend_alloc_ctx_tensors` gives an input its own buffer immediately,
//! while `ggml_gallocr` sizes every intermediate at once and reuses the space,
//! which is what keeps a deep graph from holding every layer's output at the
//! same time.
//!
//! Calling the first repeatedly is safe and is what [`Context::input_f32`] does:
//! it only allocates tensors that do not already have storage, so each input
//! gets a buffer as it is created and earlier ones are left alone.

use crate::backend::Backend;
use crate::error::{Error, Result};
use crate::sys;
use crate::tensor::Tensor;

/// A ggml context, its graph, and every buffer allocated against it.
pub struct Context<'b> {
    /// The backend every tensor here is allocated on.
    pub(crate) backend: &'b Backend,
    /// Holds tensor *metadata*; the bytes live in `buffers` or the allocator.
    pub(crate) raw: *mut sys::ggml_context,
    /// One per input tensor, freed together in `Drop`.
    ///
    /// Behind a `RefCell` so creating an input takes `&self`: several inputs
    /// have to coexist to be wired into one graph, and a `&mut self` signature
    /// would let only one live at a time. `Context` is not `Sync` — a ggml
    /// backend holds one command queue — so a cell is the whole cost.
    pub(crate) buffers: std::cell::RefCell<Vec<sys::ggml_backend_buffer_t>>,
    /// Sizes and reuses storage for the graph's intermediates.
    allocr: sys::ggml_gallocr_t,
}

impl<'b> Context<'b> {
    /// A context with room for `nodes` tensors and one graph.
    ///
    /// `nodes` is a ceiling on graph size, not an allocation of that many
    /// tensors — ggml needs the metadata arena sized up front. A BERT block
    /// costs roughly thirty nodes, so a twelve-layer model wants a few hundred.
    pub fn new(backend: &'b Backend, nodes: usize) -> Result<Self> {
        // SAFETY: plain-data params; the returned context is null-checked.
        let raw = unsafe {
            let overhead = sys::ggml_tensor_overhead()
                .saturating_mul(nodes)
                .saturating_add(sys::ggml_graph_overhead())
                .saturating_add(sys::ggml_graph_overhead());
            sys::ggml_init(sys::ggml_init_params {
                mem_size: overhead,
                mem_buffer: std::ptr::null_mut(),
                no_alloc: true,
            })
        };
        if raw.is_null() {
            return Err(Error::Allocation { bytes: 0 });
        }

        // SAFETY: `raw` is non-null; the buffer type belongs to a live backend.
        let allocr = unsafe {
            sys::ggml_gallocr_new(sys::ggml_backend_get_default_buffer_type(backend.raw()))
        };
        if allocr.is_null() {
            // SAFETY: `raw` is live and unfreed on this path.
            unsafe { sys::ggml_free(raw) };
            return Err(Error::Allocation { bytes: 0 });
        }

        Ok(Self {
            backend,
            raw,
            buffers: std::cell::RefCell::new(Vec::new()),
            allocr,
        })
    }

    /// Reference a loaded weight by name, as a node in this graph.
    ///
    /// Copies nothing: the tensor stays where [`Weights`] put it and the graph
    /// simply points at it. That is why a model is loaded once and reused
    /// across queries rather than per call — rule 45's "nothing swaps per
    /// call", made cheap rather than merely mandated.
    ///
    /// `weights` must outlive this context, which the borrow enforces.
    ///
    /// [`Weights`]: crate::Weights
    pub fn weight<'w: 'b>(
        &self,
        weights: &'w crate::Weights,
        name: &str,
    ) -> Result<Tensor<'_, 'b>> {
        match weights.raw_tensor(name) {
            Some(raw) => Ok(Tensor::new(self, raw)),
            None => Err(Error::Runtime {
                op: "weight",
                reason: format!("the model carries no tensor named {name:?}"),
            }),
        }
    }

    /// Run the graph ending at `output` and read it back as `f32`.
    ///
    /// The graph's intermediates are sized and reused by this context's
    /// allocator, so two computes on one context share that scratch space —
    /// which is safe because `Context` is not `Sync` and cannot be driven from
    /// two threads at once.
    pub fn compute(&self, output: &Tensor<'_, 'b>) -> Result<Vec<f32>> {
        let raw = output.raw();
        let len = output.elements();

        // SAFETY: every pointer below belongs to this context or its backend
        // and is live; `graph` and `allocr` are null-checked before use, and the
        // read is exactly the output tensor's element count.
        unsafe {
            let graph = sys::ggml_new_graph(self.raw);
            if graph.is_null() {
                return Err(Error::Allocation { bytes: 0 });
            }
            sys::ggml_build_forward_expand(graph, raw);

            if !sys::ggml_gallocr_alloc_graph(self.allocr, graph) {
                return Err(Error::Allocation { bytes: 0 });
            }

            let status = sys::ggml_backend_graph_compute(self.backend.raw(), graph);
            if status != sys::ggml_status_GGML_STATUS_SUCCESS {
                return Err(Error::Runtime {
                    op: "graph_compute",
                    reason: format!("ggml returned status {status}"),
                });
            }

            let mut out = vec![0f32; len];
            sys::ggml_backend_tensor_get(raw, out.as_mut_ptr().cast(), 0, len * 4);
            Ok(out)
        }
    }
}

impl Drop for Context<'_> {
    fn drop(&mut self) {
        // SAFETY: each was produced here and is freed exactly once. Buffers and
        // the allocator go before the context that describes them.
        unsafe {
            sys::ggml_gallocr_free(self.allocr);
            for buffer in self.buffers.get_mut().drain(..) {
                sys::ggml_backend_buffer_free(buffer);
            }
            sys::ggml_free(self.raw);
        }
    }
}
