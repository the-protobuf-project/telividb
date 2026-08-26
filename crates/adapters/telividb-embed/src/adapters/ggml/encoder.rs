//! A resident model, and one forward pass over a padded batch.
//!
//! The weights are loaded once and stay on the device; each call builds a graph
//! that references them and computes it. That is rule 45 — models resident,
//! nothing swapping per call — made cheap rather than merely mandated, because
//! referencing a weight in a graph copies nothing.

use super::block::Block;
use super::config::Config;
use crate::domain::Pooling;
use crate::error::{Error, Result};
use std::path::Path;
use telividb_compute::{Backend, Context, Tensor, Weights};

/// Graph nodes to reserve per block, with headroom.
///
/// ggml sizes its metadata arena up front, so this is a ceiling rather than an
/// allocation. A block records roughly thirty nodes; forty leaves room for the
/// gated feed-forward path without re-tuning per architecture.
const NODES_PER_BLOCK: usize = 40;

/// A GGUF encoder resident on a backend.
pub struct Encoder {
    weights: Weights,
    backend: Backend,
    pub(super) config: Config,
}

impl Encoder {
    /// Load `path` onto `backend` and read its shape.
    pub fn load(path: &Path, backend: Backend) -> Result<Self> {
        let weights = Weights::load(path, &backend).map_err(|e| Error::Compute(e.to_string()))?;
        let config = Config::from_weights(&weights)?;
        Ok(Self {
            weights,
            backend,
            config,
        })
    }

    /// The model's shape, for callers that need its width or context.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Encode one padded batch to a pooled vector per row.
    ///
    /// `ids` is `rows * width` token ids and `attention` the matching 0/1 mask.
    /// Returns one `hidden`-wide vector per row, in input order.
    pub fn forward(
        &self,
        ids: &[u32],
        attention: &[u32],
        rows: usize,
        pooling: Pooling,
    ) -> Result<Vec<Vec<f32>>> {
        let width = match rows {
            0 => return Ok(Vec::new()),
            _ => ids.len() / rows,
        };
        let nodes = self.config.layers * NODES_PER_BLOCK + 64;
        let ctx = Context::new(&self.backend, nodes).map_err(|e| Error::Compute(e.to_string()))?;

        let hidden = self.graph(&ctx, ids, attention, rows, width)?;
        let raw = ctx
            .compute(&hidden)
            .map_err(|e| Error::Compute(e.to_string()))?;

        Ok(self.pool(&raw, attention, rows, width, pooling))
    }

    /// Record embeddings, every block, and the final norm.
    fn graph<'c, 'b>(
        &'b self,
        ctx: &'c Context<'b>,
        ids: &[u32],
        attention: &[u32],
        rows: usize,
        width: usize,
    ) -> Result<Tensor<'c, 'b>> {
        let tokens = rows * width;
        let signed: Vec<i32> = ids.iter().map(|&i| i as i32).collect();
        let id_tensor = ctx
            .input_i32(&signed, [tokens, 1])
            .map_err(|e| Error::Compute(e.to_string()))?;

        // Additive and applied *before* every softmax, which is the only
        // correct order — zeroing weights afterwards leaves the row normalized
        // over padding, so every real token is scaled down by whatever fraction
        // of the row was padding.
        //
        // A large negative rather than -inf: an all-padding row would otherwise
        // make the softmax denominator NaN and poison the whole batch.
        //
        // Shaped (width, width, 1, rows): one key axis, one query axis, and a
        // row axis that ggml broadcasts over heads. Every query in a row sees
        // the same key mask, and no query sees another row at all.
        let mut values = Vec::with_capacity(width * width * rows);
        for row in 0..rows {
            for _query in 0..width {
                for key in 0..width {
                    values.push(match attention[row * width + key] {
                        0 => -1e9,
                        _ => 0.0,
                    });
                }
            }
        }
        let mask = ctx
            .input_f32(&values, [width, width * rows])
            .map_err(|e| Error::Compute(e.to_string()))?
            .reshape_4d(width, width, 1, rows)?;

        let table = ctx
            .weight(&self.weights, "token_embd.weight")
            .map_err(|e| Error::Compute(e.to_string()))?;
        let mut xs = table.rows(&id_tensor)?;

        // A learned position table where the model carries one; RoPE otherwise,
        // applied inside attention. Never both, and never neither.
        let positions = match self.config.uses_rope() {
            // One entry per position *within a row*: rope is applied while
            // the token axis is `ne[2] == width`, and ggml asserts the two
            // match.
            true => Some(
                ctx.input_i32(&(0..width as i32).collect::<Vec<_>>(), [width, 1])
                    .map_err(|e| Error::Compute(e.to_string()))?,
            ),
            false => {
                if let Ok(table) = ctx.weight(&self.weights, "position_embd.weight") {
                    let at = ctx
                        .input_i32(
                            &(0..tokens).map(|p| (p % width) as i32).collect::<Vec<_>>(),
                            [tokens, 1],
                        )
                        .map_err(|e| Error::Compute(e.to_string()))?;
                    xs = xs.add(&table.rows(&at)?)?;
                }
                None
            }
        };

        // Segment embedding. Single-segment input, so every token is type 0 —
        // but the trained row is not all zeros, and dropping it shifts every
        // activation in the model. Measured against the reference encoder,
        // omitting it cost ~0.17 of cosine agreement on its own.
        if let Ok(types) = ctx.weight(&self.weights, "token_types.weight") {
            let zeros = ctx
                .input_i32(&vec![0i32; tokens], [tokens, 1])
                .map_err(|e| Error::Compute(e.to_string()))?;
            xs = xs.add(&types.rows(&zeros)?)?;
        }

        if let Ok(norm_w) = ctx.weight(&self.weights, "token_embd_norm.weight")
            && let Ok(norm_b) = ctx.weight(&self.weights, "token_embd_norm.bias")
        {
            xs = xs.layer_norm(&norm_w, &norm_b, self.config.eps)?;
        }

        for i in 0..self.config.layers {
            let block = Block::new(&self.weights, i);
            xs = block.forward(ctx, &xs, &mask, positions.as_ref(), &self.config, width)?;
        }
        Ok(xs)
    }
}
