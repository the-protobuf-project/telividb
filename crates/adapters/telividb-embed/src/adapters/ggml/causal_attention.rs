//! Grouped-query self-attention, for the Qwen3 and Llama families.
//!
//! Two differences from `attention.rs`, both about shape rather than arithmetic:
//!
//! - **Fewer key heads than query heads.** Keys and values are projected at
//!   `kv_heads` and tiled up to `heads` before the scores matmul.
//! - **Heads may be wider than `hidden / heads`.** Qwen3-Embedding-0.6B is 1024
//!   wide with 16 heads of 128, so the projections are deliberately larger than
//!   the residual stream. Every reshape below reads `config.head_dim()`.

use super::block::Block;
use super::config::Config;
use crate::error::Result;
use telividb_compute::{Context, Tensor};

impl<'w> Block<'w> {
    /// Record grouped-query self-attention: project, norm, rotate, tile, score.
    pub(super) fn attention_gqa<'c, 'b>(
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
        let tokens = xs.dim(1);
        let rows = tokens / width.max(1);
        let (head_dim, heads, kv_heads) = (config.head_dim(), config.heads, config.kv_heads);

        // Queries at the full head count; keys and values at the shared one.
        let q = self.linear(ctx, xs, "attn_q")?;
        let k = self.linear(ctx, xs, "attn_k")?;
        let v = self.linear(ctx, xs, "attn_v")?;

        let q = q.reshape_4d(head_dim, heads, width, rows)?;
        let k = k.reshape_4d(head_dim, kv_heads, width, rows)?;
        let v = v.reshape_4d(head_dim, kv_heads, width, rows)?;

        // Qwen3 normalizes each head's queries and keys before rotating them.
        // Absent in Llama, so a missing tensor is the signal rather than an
        // error — and applying it where it does not belong would rescale every
        // logit.
        let q = self.head_norm(ctx, &q, "attn_q_norm", config.eps)?;
        let k = self.head_norm(ctx, &k, "attn_k_norm", config.eps)?;

        // Rotate before permuting: ggml's rope asserts that the token count is
        // the third dimension, and the permute below puts heads there. The
        // wrong order aborts the process rather than returning an error.
        let (q, k) = match (positions, config.rope_base) {
            (Some(positions), Some(base)) => (
                q.rope(positions, head_dim, base)?,
                k.rope(positions, head_dim, base)?,
            ),
            _ => (q, k),
        };

        // Heads move outward so each head's score matrix is independent:
        // (head_dim, heads, width, rows) -> (head_dim, width, heads, rows)
        let lay_out =
            |t: &Tensor<'c, 'b>| -> Result<Tensor<'c, 'b>> { Ok(t.permute(0, 2, 1, 3)?.cont()?) };
        let (q, k, v) = (lay_out(&q)?, lay_out(&k)?, lay_out(&v)?);

        // **The shared heads are not expanded; `mul_mat` pairs them.** With
        // heads on `ne[2]`, ggml broadcasts the smaller operand by an integer
        // factor — `i02 = i12 / (heads / kv_heads)` — which is exactly the
        // grouped-query pairing: query head `i` reads key head `i / ratio`.
        //
        // Tiling them explicitly with `repeat` is the obvious-looking thing and
        // it is wrong: `ggml_repeat` produces `[k0..k7, k0..k7]`, so query head
        // 1 would read key head 1 where it must read key head 0. That mistake
        // ranks an unrelated sentence above a paraphrase while every vector
        // stays finite and correctly shaped.
        let scores = k.matmul(&q)?;
        let weights = scores.masked_softmax(Some(mask), config.scale())?;
        let context = v.transpose()?.cont()?.matmul(&weights)?;

        // Back to (heads * head_dim, tokens) for the output projection, which
        // is `hidden`-wide on its far side even when the heads are not.
        let merged =
            context
                .permute(0, 2, 1, 3)?
                .cont()?
                .reshape_3d(heads * head_dim, tokens, 1)?;

        self.linear(ctx, &merged, "attn_output")
    }

    /// Normalize each head's vectors, when the file carries the weight.
    ///
    /// Qwen3 carries `attn_q_norm` and `attn_k_norm`; Llama carries neither.
    /// Absence is the architecture speaking rather than a fault, so it returns
    /// the input untouched — the same way `linear` treats an absent bias.
    fn head_norm<'c, 'b>(
        &self,
        ctx: &'c Context<'b>,
        xs: &Tensor<'c, 'b>,
        name: &str,
        eps: f32,
    ) -> Result<Tensor<'c, 'b>>
    where
        'w: 'b,
    {
        match self.tensor(ctx, &format!("{name}.weight")) {
            Ok(weight) => Ok(xs.rms_norm(&weight, eps)?),
            Err(_) => Ok(*xs),
        }
    }
}
