//! The query/key/value projection, however the file stores it.

use super::linear::QLinear;
use crate::adapters::candle::weights::Weights;
use crate::error::Result;
use candle_core::{D, Tensor};

/// How a model writes its attention input projection.
///
/// Two shapes exist in the wild and they are not distinguishable by anything
/// except which tensors are present, so the choice is made at load time from
/// the file rather than from the architecture name.
pub enum Qkv {
    /// One `[3 * hidden, hidden]` matrix. What `nomic-bert` writes, and what
    /// llama.cpp's converter produces as `attn_qkv`.
    ///
    /// Fused because all three projections read the same input: one matmul
    /// against a triple-width matrix is materially faster than three against
    /// single-width ones.
    Fused(QLinear),
    /// Three separate matrices, as classic BERT writes them.
    Split {
        /// Projects the token each position is asking *from*.
        query: QLinear,
        /// Projects what each position offers to be matched against.
        key: QLinear,
        /// Projects the content actually carried forward once a
        /// position has been attended to.
        value: QLinear,
    },
}

impl Qkv {
    /// Load block `i`'s projection, preferring the fused form when present.
    pub fn load(weights: &mut Weights, i: usize) -> Result<Self> {
        if let Ok(fused) = load_linear(weights, &format!("blk.{i}.attn_qkv")) {
            return Ok(Qkv::Fused(fused));
        }
        Ok(Qkv::Split {
            query: load_linear(weights, &format!("blk.{i}.attn_q"))?,
            key: load_linear(weights, &format!("blk.{i}.attn_k"))?,
            value: load_linear(weights, &format!("blk.{i}.attn_v"))?,
        })
    }

    /// Project `xs` into query, key and value, each `[.., hidden]`.
    pub fn project(&self, xs: &Tensor, hidden: usize) -> Result<(Tensor, Tensor, Tensor)> {
        match self {
            Qkv::Split { query, key, value } => {
                Ok((query.forward(xs)?, key.forward(xs)?, value.forward(xs)?))
            }
            Qkv::Fused(fused) => {
                // Order is q, k, v along the output axis. Getting it wrong
                // runs fine and attends to the wrong thing everywhere.
                let all = fused.forward(xs)?;
                Ok((
                    all.narrow(D::Minus1, 0, hidden)?,
                    all.narrow(D::Minus1, hidden, hidden)?,
                    all.narrow(D::Minus1, 2 * hidden, hidden)?,
                ))
            }
        }
    }
}

/// Load a `<prefix>.weight` / optional `<prefix>.bias` pair.
///
/// The bias really is optional here: nomic-bert's attention carries none at
/// all, where classic BERT carries one on every projection.
fn load_linear(weights: &mut Weights, prefix: &str) -> Result<QLinear> {
    let weight = weights.quantized(&format!("{prefix}.weight"))?;
    let bias = weights.optional(&format!("{prefix}.bias"));
    Ok(QLinear::new(weight, bias)?)
}
