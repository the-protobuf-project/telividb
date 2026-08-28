//! Building and running one ggml graph.
//!
//! Split from `score.rs` because that file is *what a score is* and this is the
//! machinery that produces one. The separation earns itself here: every early
//! return below has to free a context, an allocator and a buffer, and a missed
//! one is a leak that only shows under sustained load — so the releases live in
//! a single `Drop` rather than scattered through the construction path.

use crate::corpus::Corpus;
use crate::error::{Error, Result};
use crate::score::Scores;
use crate::sys;

/// One graph and the resources it needs, released together.
///
/// A struct rather than a long function because every early return has to free
/// the context, the allocator and the buffer — and a missed one is a leak that
/// only shows under sustained load.
pub(crate) struct GraphRun<'a> {
    corpus: &'a Corpus,
    ctx: *mut sys::ggml_context,
    allocr: sys::ggml_gallocr_t,
    buffer: sys::ggml_backend_buffer_t,
    result: *mut sys::ggml_tensor,
    graph: *mut sys::ggml_cgraph,
    queries: usize,
}

impl<'a> GraphRun<'a> {
    /// Build the graph and upload the queries.
    pub(crate) fn new(corpus: &'a Corpus, queries: &[f32], count: usize) -> Result<Self> {
        let dim = corpus.dim();

        // Metadata for the query tensor, the result, and the graph itself.
        // SAFETY: plain-data params; the context is null-checked below.
        let ctx = unsafe {
            let overhead = sys::ggml_tensor_overhead()
                .saturating_mul(8)
                .saturating_add(sys::ggml_graph_overhead());
            sys::ggml_init(sys::ggml_init_params {
                mem_size: overhead,
                mem_buffer: std::ptr::null_mut(),
                no_alloc: true,
            })
        };
        if ctx.is_null() {
            return Err(Error::Allocation { bytes: 0 });
        }

        match Self::build(corpus, ctx, queries, count, dim) {
            Ok((allocr, buffer, result, graph)) => Ok(Self {
                corpus,
                ctx,
                allocr,
                buffer,
                result,
                graph,
                queries: count,
            }),
            Err(e) => {
                // SAFETY: `ctx` is live and unfreed on this path.
                unsafe { sys::ggml_free(ctx) };
                Err(e)
            }
        }
    }

    /// Wire the query tensor and the matmul into a graph.
    #[allow(clippy::type_complexity)]
    fn build(
        corpus: &Corpus,
        ctx: *mut sys::ggml_context,
        queries: &[f32],
        count: usize,
        dim: usize,
    ) -> Result<(
        sys::ggml_gallocr_t,
        sys::ggml_backend_buffer_t,
        *mut sys::ggml_tensor,
        *mut sys::ggml_cgraph,
    )> {
        // SAFETY: `ctx` has room for these tensors' metadata, and every
        // dimension is non-zero by the caller's checks. Each pointer is
        // null-checked before it is used.
        unsafe {
            let q = sys::ggml_new_tensor_2d(
                ctx,
                sys::ggml_type_GGML_TYPE_F32,
                dim as i64,
                count as i64,
            );
            if q.is_null() {
                return Err(Error::Allocation {
                    bytes: count * dim * 4,
                });
            }

            let buffer = sys::ggml_backend_alloc_ctx_tensors(ctx, corpus.backend().raw());
            if buffer.is_null() {
                return Err(Error::Allocation {
                    bytes: count * dim * 4,
                });
            }
            sys::ggml_backend_tensor_set(
                q,
                queries.as_ptr().cast(),
                0,
                std::mem::size_of_val(queries),
            );

            // (dim, rows) x (dim, count) -> (rows, count)
            let result = sys::ggml_mul_mat(ctx, corpus.tensor(), q);
            let graph = sys::ggml_new_graph(ctx);
            if result.is_null() || graph.is_null() {
                sys::ggml_backend_buffer_free(buffer);
                return Err(Error::Runtime {
                    op: "mul_mat",
                    reason: "ggml refused to build the graph".to_owned(),
                });
            }
            sys::ggml_build_forward_expand(graph, result);

            // The allocator sizes the intermediates the graph needs; the
            // corpus and query tensors already have their own storage.
            let allocr = sys::ggml_gallocr_new(sys::ggml_backend_get_default_buffer_type(
                corpus.backend().raw(),
            ));
            if allocr.is_null() || !sys::ggml_gallocr_alloc_graph(allocr, graph) {
                sys::ggml_backend_buffer_free(buffer);
                return Err(Error::Allocation { bytes: 0 });
            }

            Ok((allocr, buffer, result, graph))
        }
    }

    /// Run the graph and read the scores back.
    pub(crate) fn compute(&self) -> Result<Scores> {
        let rows = self.corpus.rows();
        let mut values = vec![0f32; self.queries * rows];

        // SAFETY: the graph was allocated against this backend, and the read is
        // exactly the result tensor's size — `queries * rows` f32 by
        // construction of the matmul above.
        unsafe {
            let status = sys::ggml_backend_graph_compute(self.corpus.backend().raw(), self.graph);
            if status != sys::ggml_status_GGML_STATUS_SUCCESS {
                return Err(Error::Runtime {
                    op: "graph_compute",
                    reason: format!("ggml returned status {status}"),
                });
            }
            sys::ggml_backend_tensor_get(
                self.result,
                values.as_mut_ptr().cast(),
                0,
                std::mem::size_of_val(values.as_slice()),
            );
        }

        Ok(Scores::new(values, self.queries, rows))
    }
}

impl Drop for GraphRun<'_> {
    fn drop(&mut self) {
        // SAFETY: each was produced by `build` and is freed exactly once. The
        // allocator and buffer are released before the context describing them.
        unsafe {
            sys::ggml_gallocr_free(self.allocr);
            sys::ggml_backend_buffer_free(self.buffer);
            sys::ggml_free(self.ctx);
        }
    }
}
