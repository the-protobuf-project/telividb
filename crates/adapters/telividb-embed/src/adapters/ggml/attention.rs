//! Multi-head self-attention for one block.
//!
//! Split from `block.rs` because it is where every shape change in the encoder
//! happens: activations arrive `(hidden, tokens)`, are viewed as
//! `(head_dim, heads, tokens)`, and are permuted so heads become the outer axis
//! so each head's score matrix is computed independently. A bug here is a
//! layout bug, and it looks nothing like the arithmetic bugs in `block.rs`.

use super::block::Block;
use super::config::Config;
use crate::error::Result;
use telividb_compute::{Context, Tensor};

impl<'w> Block<'w> {
    /// Record this block's self-attention: project, rotate, score, mix.
    ///
    /// Returns a handle, not a value — nothing computes until the caller
    /// dispatches the whole graph.
    pub(super) fn attention<'c, 'b>(
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
        let (q, k, v) = self.project(ctx, xs, config)?;

        // (hidden, rows * width) -> (head_dim, heads, width, rows).
        //
        // The fourth axis is the batch, and keeping it separate is what stops
        // one row attending to another: every operation below broadcasts over
        // it rather than mixing across it.
        let heads = |t: &Tensor<'c, 'b>| -> Result<Tensor<'c, 'b>> {
            Ok(t.reshape_4d(config.head_dim(), config.heads, width, rows)?)
        };
        let (q, k, v) = (heads(&q)?, heads(&k)?, heads(&v)?);

        // **Rotate before permuting, not after.** ggml's rope requires the
        // token count to be the third dimension — it asserts
        // `a->ne[2] == positions->ne[0]` — and the permute below moves heads
        // there instead. Applying it in the other order aborts the process on
        // that assert rather than returning an error, so the ordering is not a
        // preference.
        //
        // Queries and keys only, never values: rotating V mixes content with
        // position and yields plausible nonsense.
        let (q, k) = match (positions, config.rope_base) {
            (Some(positions), Some(base)) => (
                q.rope(positions, config.head_dim(), base)?,
                k.rope(positions, config.head_dim(), base)?,
            ),
            _ => (q, k),
        };

        // Now heads move outward so each head's score matrix is independent:
        // (head_dim, heads, width, rows) -> (head_dim, width, heads, rows)
        let lay_out =
            |t: &Tensor<'c, 'b>| -> Result<Tensor<'c, 'b>> { Ok(t.permute(0, 2, 1, 3)?.cont()?) };
        let (q, k, v) = (lay_out(&q)?, lay_out(&k)?, lay_out(&v)?);

        // Contracts head_dim, giving (width, width, heads, rows) — one score
        // matrix per head per row.
        let scores = k.matmul(&q)?;
        // Scale and mask fused into the softmax, so the score matrix is read
        // once and the mask can only be applied *before* normalization.
        let weights = scores.masked_softmax(Some(mask), config.scale())?;

        // (width, head_dim, heads, rows) x (width, width, heads, rows)
        //   -> (head_dim, width, heads, rows)
        let context = v.transpose()?.cont()?.matmul(&weights)?;
        let merged = context
            .permute(0, 2, 1, 3)?
            .cont()?
            .reshape_3d(config.hidden, tokens, 1)?;

        self.linear(ctx, &merged, "attn_output")
    }

    /// Query, key and value, from whichever projection layout the file carries.
    #[allow(clippy::type_complexity)]
    fn project<'c, 'b>(
        &self,
        ctx: &'c Context<'b>,
        xs: &Tensor<'c, 'b>,
        config: &Config,
    ) -> Result<(Tensor<'c, 'b>, Tensor<'c, 'b>, Tensor<'c, 'b>)>
    where
        'w: 'b,
    {
        // Fused when present — what nomic-bert and llama.cpp's converter write.
        // One matmul against a triple-width matrix beats three narrow ones,
        // since all three read the same input.
        if let Ok(all) = self.linear(ctx, xs, "attn_qkv") {
            // Order is q, k, v along the output axis — llama.cpp's convention
            // for `attn_qkv`. Getting it wrong runs fine and attends to the
            // wrong thing everywhere.
            let h = config.hidden;
            return Ok((
                all.chunk(0, h)?.cont()?,
                all.chunk(1, h)?.cont()?,
                all.chunk(2, h)?.cont()?,
            ));
        }

        Ok((
            self.linear(ctx, xs, "attn_q")?,
            self.linear(ctx, xs, "attn_k")?,
            self.linear(ctx, xs, "attn_v")?,
        ))
    }
}
