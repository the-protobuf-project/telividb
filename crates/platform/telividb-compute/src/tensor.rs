//! A handle into a graph, and the operations that extend it.
//!
//! Every method here **records a node and returns a handle** — none of them
//! compute. The work happens in [`Context::compute`], once, for the whole
//! graph. Writing `a.matmul(b)` and expecting a value back is the most natural
//! mistake to make against this API, which is why the return type is `Tensor`
//! rather than anything resembling data.
//!
//! # Dimension order
//!
//! ggml numbers dimensions **fastest-varying first**, the reverse of how most
//! array libraries print a shape. `ne[0]` is the contiguous axis. A matrix of
//! `tokens` vectors each `dim` wide is `(dim, tokens)`, one vector per column.
//!
//! [`Tensor::matmul`] follows from that: `a.matmul(b)` contracts `ne[0]`, the
//! dimension the two share, and produces `(a.ne[1], b.ne[1])`.

use crate::context::Context;
use crate::error::{Error, Result};
use crate::sys;

/// A node in the graph being built.
///
/// Borrows its context, so a tensor cannot outlive the graph it belongs to —
/// which is what stops a handle being used after the arena behind it is freed.
#[derive(Clone, Copy)]
pub struct Tensor<'c, 'b> {
    ctx: &'c Context<'b>,
    raw: *mut sys::ggml_tensor,
}

impl<'c, 'b> Tensor<'c, 'b> {
    /// Wrap a raw node. Internal: a `Tensor` is only ever produced by the
    /// context or by an operation on another tensor.
    pub(crate) fn new(ctx: &'c Context<'b>, raw: *mut sys::ggml_tensor) -> Self {
        Self { ctx, raw }
    }

    /// The underlying node, for operations in this crate only.
    pub(crate) fn raw(&self) -> *mut sys::ggml_tensor {
        self.raw
    }

    /// Extent along `axis`, which must be 0..4.
    pub fn dim(&self, axis: usize) -> usize {
        // SAFETY: `raw` is a live node; `ne` is a fixed four-element array.
        match axis < 4 {
            true => unsafe { (*self.raw).ne[axis] as usize },
            false => 1,
        }
    }

    /// Total element count across every axis.
    pub fn elements(&self) -> usize {
        (0..4).map(|axis| self.dim(axis)).product()
    }

    /// The context this node belongs to, for the sibling operation modules.
    pub(crate) fn ctx(&self) -> &'c Context<'b> {
        self.ctx
    }

    /// Record `op` and wrap the result, failing if ggml refused it.
    pub(crate) fn wrap(&self, op: &'static str, raw: *mut sys::ggml_tensor) -> Result<Self> {
        match raw.is_null() {
            true => Err(Error::Runtime {
                op,
                reason: "ggml refused the operation — check the operand shapes".to_owned(),
            }),
            false => Ok(Tensor::new(self.ctx, raw)),
        }
    }

    /// Matrix product contracting the shared `ne[0]`.
    ///
    /// `self` is the weight and `other` the activation, matching ggml's own
    /// convention: `(k, n) x (k, m) -> (n, m)`. A quantized `self` is handled
    /// natively — ggml multiplies Q4_K or Q8_0 weights against f32 activations
    /// without a dequantization pass, which is the whole reason a GGUF model
    /// runs at its stated memory footprint rather than at f32 size.
    pub fn matmul(&self, other: &Self) -> Result<Self> {
        // SAFETY: both nodes belong to this context, which is live.
        self.wrap("mul_mat", unsafe {
            sys::ggml_mul_mat(self.ctx.raw, self.raw, other.raw)
        })
    }

    /// Elementwise sum, broadcasting `other` where its extents are 1.
    pub fn add(&self, other: &Self) -> Result<Self> {
        // SAFETY: both nodes belong to this context, which is live.
        self.wrap("add", unsafe {
            sys::ggml_add(self.ctx.raw, self.raw, other.raw)
        })
    }

    /// Elementwise product, broadcasting `other` where its extents are 1.
    pub fn mul(&self, other: &Self) -> Result<Self> {
        // SAFETY: both nodes belong to this context, which is live.
        self.wrap("mul", unsafe {
            sys::ggml_mul(self.ctx.raw, self.raw, other.raw)
        })
    }

    /// Multiply every element by `factor`.
    pub fn scale(&self, factor: f32) -> Result<Self> {
        // SAFETY: `raw` is a live node in this context.
        self.wrap("scale", unsafe {
            sys::ggml_scale(self.ctx.raw, self.raw, factor)
        })
    }

    /// Gather rows of `self` selected by `ids`.
    ///
    /// This is the embedding lookup: `self` is the vocabulary table and `ids`
    /// an `i32` tensor of token ids.
    pub fn rows(&self, ids: &Self) -> Result<Self> {
        // SAFETY: both nodes belong to this context, which is live.
        self.wrap("get_rows", unsafe {
            sys::ggml_get_rows(self.ctx.raw, self.raw, ids.raw)
        })
    }

    /// Normalize over `ne[0]`, then scale by `weight` and shift by `bias`.
    ///
    /// The composed form rather than a single op because ggml's `norm` does the
    /// normalization only — the affine half is two more nodes, and forgetting
    /// them yields activations that are finite, correctly shaped and wrong.
    pub fn layer_norm(&self, weight: &Self, bias: &Self, eps: f32) -> Result<Self> {
        // SAFETY: every node belongs to this context, which is live.
        let normed = self.wrap("norm", unsafe {
            sys::ggml_norm(self.ctx.raw, self.raw, eps)
        })?;
        normed.mul(weight)?.add(bias)
    }

    /// Normalize over `ne[0]` by root-mean-square, then scale by `weight`.
    ///
    /// The normalization every current transformer uses in place of
    /// [`layer_norm`](Self::layer_norm), and the difference is not only
    /// cosmetic: RMS norm subtracts no mean and adds no bias, so the two are
    /// not interchangeable. Applying the wrong one produces activations that
    /// are finite and correctly shaped, and a model that quietly returns worse
    /// vectors — the failure mode this crate's shape checks cannot catch.
    ///
    /// No bias parameter, because these architectures have none. A weight is
    /// still required: ggml's `rms_norm` does the normalization only, and the
    /// scale is a separate node.
    pub fn rms_norm(&self, weight: &Self, eps: f32) -> Result<Self> {
        // SAFETY: every node belongs to this context, which is live.
        let normed = self.wrap("rms_norm", unsafe {
            sys::ggml_rms_norm(self.ctx.raw, self.raw, eps)
        })?;
        normed.mul(weight)
    }

    /// Softmax over `ne[0]`.
    pub fn softmax(&self) -> Result<Self> {
        // SAFETY: `raw` is a live node in this context.
        self.wrap("soft_max", unsafe {
            sys::ggml_soft_max(self.ctx.raw, self.raw)
        })
    }

    /// The exact GELU, via the error function.
    ///
    /// BERT-family weights were trained against the erf form; the `tanh`
    /// approximation differs by enough to move rankings while leaving every
    /// vector well-formed.
    pub fn gelu(&self) -> Result<Self> {
        // SAFETY: `raw` is a live node in this context.
        self.wrap("gelu_erf", unsafe {
            sys::ggml_gelu_erf(self.ctx.raw, self.raw)
        })
    }

    /// SiLU, the gate activation in a SwiGLU feed-forward network.
    pub fn silu(&self) -> Result<Self> {
        // SAFETY: `raw` is a live node in this context.
        self.wrap("silu", unsafe { sys::ggml_silu(self.ctx.raw, self.raw) })
    }
}

#[cfg(test)]
#[path = "tensor_test.rs"]
mod tests;
