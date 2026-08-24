//! The assembled encoder.

use super::block::Block;
use super::embeddings::Embeddings;
use crate::adapters::candle::config::BertConfig;
use crate::adapters::candle::weights::Weights;
use crate::domain::Pooling;
use crate::error::Result;
use candle_core::{D, DType, Device, Tensor};

/// A GGUF-backed BERT encoder, resident on one device.
pub struct QuantizedBert {
    embeddings: Embeddings,
    blocks: Vec<Block>,
    config: BertConfig,
    device: Device,
}

impl QuantizedBert {
    /// Load every tensor from an open GGUF.
    ///
    /// Eager, not lazy: rule 45 holds models resident, so paying the load cost
    /// once at registration is the point. A lazily-loaded layer would also put
    /// a file read inside the first inference, where the caller is waiting.
    pub fn load(weights: &mut Weights) -> Result<Self> {
        let config = BertConfig::from_gguf(weights.content())?;
        let embeddings = Embeddings::load(weights, &config)?;
        let blocks = (0..config.layers)
            .map(|i| Block::load(weights, &config, i))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            embeddings,
            blocks,
            device: weights.device().clone(),
            config,
        })
    }

    /// The width of the vectors this model produces.
    pub fn hidden(&self) -> usize {
        self.config.hidden
    }

    /// Longest sequence the position embeddings cover.
    pub fn context(&self) -> usize {
        self.config.context
    }

    /// The architecture this model declares, e.g. `nomic-bert`.
    pub fn architecture(&self) -> &str {
        &self.config.arch
    }

    /// Width of the feed-forward intermediate projection.
    pub fn feed_forward(&self) -> usize {
        self.config.ff
    }

    /// Where this model is resident.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Encode a padded batch to one vector per row.
    ///
    /// `ids` and `attention` are both `[batch, seq]`; `attention` is 1 for a
    /// real token and 0 for padding.
    pub fn forward(&self, ids: &Tensor, attention: &Tensor, pooling: Pooling) -> Result<Tensor> {
        let mut xs = self.embeddings.forward(ids, &self.device)?;
        let mask = additive_mask(attention)?;
        for block in &self.blocks {
            xs = block.forward(&xs, &mask)?;
        }
        Ok(pool(&xs, attention, pooling)?)
    }
}

/// Turn a 0/1 attention mask into the additive form attention wants.
///
/// A large negative rather than `-inf`: `-inf` times a zero score is `NaN`,
/// and a row that is entirely padding — which a short text in a long batch
/// produces — would poison the whole batch through the softmax denominator.
fn additive_mask(attention: &Tensor) -> candle_core::Result<Tensor> {
    let (batch, seq) = attention.dims2()?;
    let keep = attention.to_dtype(DType::F32)?;
    ((keep - 1.0)? * 1e9)?.reshape((batch, 1, 1, seq))
}

/// Collapse `[batch, seq, hidden]` to `[batch, hidden]`.
fn pool(xs: &Tensor, attention: &Tensor, pooling: Pooling) -> candle_core::Result<Tensor> {
    match pooling {
        Pooling::Cls => xs.i((.., 0))?.contiguous(),
        Pooling::Mean => {
            // Weighted by the mask so padding contributes nothing, and divided
            // by the real token count rather than the padded width — otherwise
            // a short text in a long batch is scaled down by however much
            // padding it happened to receive.
            let keep = attention.to_dtype(DType::F32)?.unsqueeze(D::Minus1)?;
            let summed = xs.broadcast_mul(&keep)?.sum(1)?;
            let count = keep.sum(1)?.clamp(1.0, f32::INFINITY)?;
            summed.broadcast_div(&count)
        }
    }
}

use candle_core::IndexOp;

#[cfg(test)]
#[path = "encode_test.rs"]
mod tests;
