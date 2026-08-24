//! Token, position and type embeddings, summed and normalized.

use super::ops::layer_norm;
use crate::adapters::candle::config::BertConfig;
use crate::adapters::candle::weights::Weights;
use crate::error::Result;
use candle_core::quantized::QTensor;
use candle_core::{Device, Tensor};
use std::sync::Arc;

/// The input layer: three lookups, summed, then normalized.
pub struct Embeddings {
    tokens: Arc<QTensor>,
    /// Absent for a rotary model, where position is applied inside attention
    /// rather than added here.
    positions: Option<Arc<QTensor>>,
    /// Segment embedding. Present in BERT proper, absent in `nomic-bert`,
    /// which dropped the next-sentence objective it served.
    types: Option<Arc<QTensor>>,
    norm_weight: Tensor,
    norm_bias: Tensor,
    eps: f64,
}

impl Embeddings {
    /// Load from an open GGUF.
    pub fn load(weights: &mut Weights, config: &BertConfig) -> Result<Self> {
        let types = weights.quantized("token_types.weight").ok();
        Ok(Self {
            tokens: weights.quantized("token_embd.weight")?,
            positions: match config.uses_rope() {
                true => None,
                false => Some(weights.quantized("position_embd.weight")?),
            },
            types,
            norm_weight: weights.dequantized("token_embd_norm.weight")?,
            norm_bias: weights.dequantized("token_embd_norm.bias")?,
            eps: config.eps,
        })
    }

    /// Embed `ids`, shaped `[batch, seq]`, into `[batch, seq, hidden]`.
    pub fn forward(&self, ids: &Tensor, device: &Device) -> Result<Tensor> {
        let (_, seq) = ids.dims2()?;
        let mut xs = self.tokens.embedding(ids)?;

        if let Some(positions) = &self.positions {
            // Positions are the same for every row in the batch, so the lookup
            // is done once and broadcast rather than per row.
            let steps = Tensor::arange(0u32, seq as u32, device)?;
            xs = xs.broadcast_add(&positions.embedding(&steps)?)?;
        }

        if let Some(types) = &self.types {
            // Single-segment input: every token is type 0. Kept rather than
            // skipped because the trained tensor is not all zeros, so dropping
            // it shifts every activation.
            let zeros = Tensor::zeros((seq,), candle_core::DType::U32, device)?;
            xs = xs.broadcast_add(&types.embedding(&zeros)?)?;
        }

        Ok(layer_norm(
            &xs,
            &self.norm_weight,
            &self.norm_bias,
            self.eps,
        )?)
    }
}
