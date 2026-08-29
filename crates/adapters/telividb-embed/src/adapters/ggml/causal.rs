//! One pre-norm transformer block, for the Qwen3 and Llama families.
//!
//! The same graph-building discipline as `block.rs` — nothing here computes —
//! but three things differ from BERT, and each of them silently degrades the
//! output rather than failing if it is got wrong:
//!
//! - **Pre-norm, not post-norm.** The norm is applied to the *input* of each
//!   sublayer and the residual adds the unnormalized value. BERT normalizes
//!   after the residual. Swapping them leaves a model that still produces
//!   finite vectors of the right width.
//! - **RMSNorm, not LayerNorm.** No mean subtraction, no bias.
//! - **A gated feed-forward network**, which `block.rs` already handles: it
//!   takes the SwiGLU branch whenever the file carries `ffn_gate`.

use super::block::Block;
use super::config::Config;
use crate::error::Result;
use telividb_compute::{Context, Tensor};

impl<'w> Block<'w> {
    /// Record this block's forward pass over `xs`, shaped `(hidden, tokens)`.
    pub(super) fn forward_causal<'c, 'b>(
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
        // Attention over the normalized input; the residual carries the value
        // that went *in*, not the normalized one.
        let normed = self.rms(ctx, xs, "attn_norm", config.eps)?;
        let attended = self.attention_gqa(ctx, &normed, mask, positions, config, width)?;
        let xs = xs.add(&attended)?;

        let normed = self.rms(ctx, &xs, "ffn_norm", config.eps)?;
        let ffn = self.feed_forward(ctx, &normed)?;
        Ok(xs.add(&ffn)?)
    }

    /// `weight * rms_norm(x)`, for a named norm in this block.
    ///
    /// No bias, because these architectures carry none — looking one up and
    /// adding zero would be harmless, but looking one up and *finding* a
    /// differently-purposed tensor would not.
    pub(super) fn rms<'c, 'b>(
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
        Ok(xs.rms_norm(&weight, eps)?)
    }
}
