//! One transformer block: attend, then feed forward, each residual-added and
//! normalized.

use super::attention::Attention;
use super::ffn::FeedForward;
use super::ops::layer_norm;
use super::rope::Rope;
use crate::adapters::candle::config::BertConfig;
use crate::adapters::candle::weights::Weights;
use crate::error::{Error, Result};
use candle_core::Tensor;

/// A single encoder layer.
///
/// Post-norm, which is what both BERT and nomic-bert were trained with: the
/// norm comes *after* the residual add, not before it. Pre-norm is the modern
/// default and would run without error on these weights while producing
/// different activations at every layer.
pub struct Block {
    attention: Attention,
    attn_norm_weight: Tensor,
    attn_norm_bias: Tensor,
    ffn: FeedForward,
    ff_norm_weight: Tensor,
    ff_norm_bias: Tensor,
    eps: f64,
}

impl Block {
    /// Load block `i`.
    pub fn load(weights: &mut Weights, config: &BertConfig, i: usize) -> Result<Self> {
        let name = |part: &str| format!("blk.{i}.{part}");
        let ffn = FeedForward::load(weights, i)?;

        // The header's feed-forward width and the tensor's must agree. They
        // are written independently, and a disagreement means one of the two
        // describes a different model — which would otherwise surface as a
        // shape error several layers deeper, pointing at the wrong place.
        match ffn.intermediate_width() {
            Some(found) if found != config.ff => {
                return Err(Error::MissingFromGguf {
                    what: format!(
                        "blk.{i} feed-forward sized for the declared width {}; \
                         the tensor is {found} wide",
                        config.ff
                    ),
                });
            }
            // A bias-less projection reveals no width, so there is nothing to
            // check — the matmul still catches a genuine mismatch, just with
            // a less specific message.
            _ => {}
        }

        Ok(Self {
            attention: Attention::load(weights, config, i)?,
            attn_norm_weight: weights.dequantized(&name("attn_output_norm.weight"))?,
            attn_norm_bias: weights.dequantized(&name("attn_output_norm.bias"))?,
            ffn,
            ff_norm_weight: weights.dequantized(&name("layer_output_norm.weight"))?,
            ff_norm_bias: weights.dequantized(&name("layer_output_norm.bias"))?,
            eps: config.eps,
        })
    }

    /// Run `xs` through the block under `mask`.
    pub fn forward(&self, xs: &Tensor, mask: &Tensor, rope: Option<&Rope>) -> Result<Tensor> {
        let attended = self.attention.forward(xs, mask, rope)?;
        let xs = layer_norm(
            &(attended + xs)?,
            &self.attn_norm_weight,
            &self.attn_norm_bias,
            self.eps,
        )?;

        let ff = self.ffn.forward(&xs)?;
        Ok(layer_norm(
            &(ff + &xs)?,
            &self.ff_norm_weight,
            &self.ff_norm_bias,
            self.eps,
        )?)
    }
}
