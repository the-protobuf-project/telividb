//! One transformer block, recorded into a graph.
//!
//! Nothing here computes. Each method extends the graph and returns a handle;
//! the whole stack is dispatched once, by the caller, after the last block.
//!
//! **Tensor layout.** ggml orders dimensions fastest-varying first, so
//! activations are `(hidden, tokens)` — one token per column. Attention views
//! that as `(head_dim, heads, tokens)` and permutes so heads become the outer
//! axis, which is the only reordering in the block and the place a shape bug
//! will be if there is one.

use super::config::Config;
use crate::error::{Error, Result};
use telividb_compute::{Context, Tensor, Weights};

/// The tensors one block owns, resolved by name once at load time.
pub struct Block<'w> {
    weights: &'w Weights,
    /// `blk.{i}.` — every tensor of this block shares it.
    prefix: String,
}

impl<'w> Block<'w> {
    /// Bind block `i`'s tensors.
    pub fn new(weights: &'w Weights, i: usize) -> Self {
        Self {
            weights,
            prefix: format!("blk.{i}."),
        }
    }

    /// Record this block's forward pass over `xs`, shaped `(hidden, tokens)`.
    pub fn forward<'c, 'b>(
        &self,
        ctx: &'c Context<'b>,
        xs: &Tensor<'c, 'b>,
        mask: &Tensor<'c, 'b>,
        positions: Option<&Tensor<'c, 'b>>,
        config: &Config,
        width: usize,
    ) -> Result<Tensor<'c, 'b>>
    where
        'w: 'b,
    {
        let attended = self.attention(ctx, xs, mask, positions, config, width)?;
        // Residual, then norm — post-norm, as BERT was trained. Pre-norm would
        // be a different model wearing the same weights.
        let xs = xs.add(&attended)?;
        let xs = self.norm(ctx, &xs, "attn_output_norm", config.eps)?;

        let ffn = self.feed_forward(ctx, &xs)?;
        let xs = xs.add(&ffn)?;
        self.norm(ctx, &xs, "layer_output_norm", config.eps)
    }

    /// `down(gelu(up(x)))`, or the gated form when a gate is present.
    pub(super) fn feed_forward<'c, 'b>(
        &self,
        ctx: &'c Context<'b>,
        xs: &Tensor<'c, 'b>,
    ) -> Result<Tensor<'c, 'b>>
    where
        'w: 'b,
    {
        let up = self.linear(ctx, xs, "ffn_up")?;
        // SwiGLU when the file carries a gate: activation on the *gate* branch
        // only, value branch unactivated. Applying it to both is a silent
        // accuracy loss rather than an error.
        let hidden = match self.linear(ctx, xs, "ffn_gate") {
            Ok(gate) => up.mul(&gate.silu()?)?,
            Err(_) => up.gelu()?,
        };
        self.linear(ctx, &hidden, "ffn_down")
    }

    /// `weight * norm(x) + bias`, for a named norm in this block.
    fn norm<'c, 'b>(
        &self,
        ctx: &'c Context<'b>,
        xs: &Tensor<'c, 'b>,
        name: &str,
        eps: f32,
    ) -> Result<Tensor<'c, 'b>>
    where
        'w: 'b,
    {
        let weight = self.tensor(ctx, &format!("{name}.weight"))?;
        let bias = self.tensor(ctx, &format!("{name}.bias"))?;
        Ok(xs.layer_norm(&weight, &bias, eps)?)
    }

    /// `weight x  xs (+ bias)`, for a named projection in this block.
    pub(super) fn linear<'c, 'b>(
        &self,
        ctx: &'c Context<'b>,
        xs: &Tensor<'c, 'b>,
        name: &str,
    ) -> Result<Tensor<'c, 'b>>
    where
        'w: 'b,
    {
        let weight = self.tensor(ctx, &format!("{name}.weight"))?;
        let out = weight.matmul(xs)?;
        // The bias really is optional: nomic-bert's attention carries none,
        // where classic BERT carries one on every projection.
        match self.tensor(ctx, &format!("{name}.bias")) {
            Ok(bias) => Ok(out.add(&bias)?),
            Err(_) => Ok(out),
        }
    }

    /// One of this block's tensors, by suffix.
    pub(super) fn tensor<'c, 'b>(
        &self,
        ctx: &'c Context<'b>,
        suffix: &str,
    ) -> Result<Tensor<'c, 'b>>
    where
        'w: 'b,
    {
        ctx.weight(self.weights, &format!("{}{suffix}", self.prefix))
            .map_err(|e| Error::Compute(e.to_string()))
    }
}
