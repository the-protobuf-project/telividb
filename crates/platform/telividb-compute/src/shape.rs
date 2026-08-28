//! Reshaping and reordering a tensor without touching its values.
//!
//! Attention spends most of its graph here rather than in arithmetic: the same
//! buffer is viewed as `(dim, tokens)`, then `(head_dim, heads, tokens)`, then
//! transposed so heads become the batch axis. Splitting these out of `tensor.rs`
//! keeps that file about arithmetic and this one about layout, which is the
//! seam a reader actually needs — a shape bug and a maths bug look nothing
//! alike and are found in different ways.
//!
//! **`permute` and `transpose` produce non-contiguous views.** ggml records the
//! new strides and copies nothing, which is what makes them free — but most
//! operations require contiguous input. [`Tensor::cont`] is what materializes
//! one, and forgetting it is the characteristic ggml bug: the graph builds, the
//! shapes look right, and the values are read in the wrong order.

use crate::error::Result;
use crate::sys;
use crate::tensor::Tensor;

impl Tensor<'_, '_> {
    /// View this tensor as three dimensions, fastest-varying first.
    ///
    /// The element count must be unchanged; ggml refuses otherwise.
    pub fn reshape_3d(&self, ne0: usize, ne1: usize, ne2: usize) -> Result<Self> {
        // SAFETY: `raw` is a live node in this context; ggml validates that the
        // requested extents multiply to the existing element count.
        self.wrap("reshape_3d", unsafe {
            sys::ggml_reshape_3d(
                self.ctx().raw,
                self.raw(),
                ne0 as i64,
                ne1 as i64,
                ne2 as i64,
            )
        })
    }

    /// View this tensor as four dimensions, fastest-varying first.
    ///
    /// The fourth axis is what keeps a batch's rows independent: attention over
    /// `(head_dim, heads, width, rows)` computes each row's scores separately,
    /// where a flattened `(hidden, rows * width)` would let one row attend to
    /// another unless a `tokens x tokens` mask forbade it — quadratic in the
    /// whole batch rather than in one row.
    pub fn reshape_4d(&self, ne0: usize, ne1: usize, ne2: usize, ne3: usize) -> Result<Self> {
        // SAFETY: `raw` is a live node; ggml validates that the requested
        // extents multiply to the existing element count.
        self.wrap("reshape_4d", unsafe {
            sys::ggml_reshape_4d(
                self.ctx().raw,
                self.raw(),
                ne0 as i64,
                ne1 as i64,
                ne2 as i64,
                ne3 as i64,
            )
        })
    }

    /// Reorder axes, producing a **non-contiguous view**.
    ///
    /// Each argument says where the corresponding source axis lands, so
    /// `permute(0, 2, 1, 3)` swaps axes 1 and 2. Follow with [`Tensor::cont`]
    /// before anything that needs contiguous memory.
    pub fn permute(&self, axis0: usize, axis1: usize, axis2: usize, axis3: usize) -> Result<Self> {
        // SAFETY: `raw` is a live node; ggml bounds-checks the axis indices.
        self.wrap("permute", unsafe {
            sys::ggml_permute(
                self.ctx().raw,
                self.raw(),
                axis0 as i32,
                axis1 as i32,
                axis2 as i32,
                axis3 as i32,
            )
        })
    }

    /// Swap the first two axes, producing a **non-contiguous view**.
    pub fn transpose(&self) -> Result<Self> {
        // SAFETY: `raw` is a live node in this context.
        self.wrap("transpose", unsafe {
            sys::ggml_transpose(self.ctx().raw, self.raw())
        })
    }

    /// One of `n` equal slices along `ne[0]`, as a view.
    ///
    /// The fused-QKV case: a projection writes `(3 * hidden, tokens)` in one
    /// matmul, and query, key and value are `chunk(0, hidden)`, `chunk(1, ..)`
    /// and `chunk(2, ..)`. Copies nothing — it records an offset and a stride,
    /// which is why the fused projection is worth doing at all.
    ///
    /// The result is a **view**: contiguous within each column but strided
    /// between them, so follow with [`Tensor::cont`] before anything that reads
    /// memory in order.
    pub fn chunk(&self, index: usize, width: usize) -> Result<Self> {
        let rows = self.dim(1);
        // SAFETY: `raw` is a live node; `nb[1]` is its column stride in bytes
        // and `element_size` its scalar width, both maintained by ggml. The
        // offset stays inside the tensor because `index * width` is bounded by
        // `ne[0]`, which ggml itself validates when it builds the view.
        self.wrap("view_2d", unsafe {
            let stride = (*self.raw()).nb[1];
            let offset = index * width * sys::ggml_element_size(self.raw());
            sys::ggml_view_2d(
                self.ctx().raw,
                self.raw(),
                width as i64,
                rows as i64,
                stride,
                offset,
            )
        })
    }

    /// Materialize a contiguous copy.
    ///
    /// Required after [`Tensor::permute`] or [`Tensor::transpose`] before an
    /// operation that reads memory in order. This is the one op here that
    /// actually moves bytes.
    pub fn cont(&self) -> Result<Self> {
        // SAFETY: `raw` is a live node in this context.
        self.wrap("cont", unsafe {
            sys::ggml_cont(self.ctx().raw, self.raw())
        })
    }
}
