//! One transformer block: attend, then feed forward, each residual-added and
//! normalized.

use super::attention::Attention;
use super::linear::QLinear;
use super::ops::{gelu, layer_norm};
use crate::adapters::candle::config::BertConfig;
use crate::adapters::candle::weights::Weights;
use crate::error::Result;
use candle_core::Tensor;

/// A single encoder layer.
///
/// Post-norm, which is what BERT-family weights were trained with: the norm
/// comes *after* the residual add, not before it. Pre-norm is the modern
/// default and would run without error on these weights while producing
/// different activations at every layer.
pub struct Block {
    attention: Attention,
    attn_norm_weight: Tensor,
    attn_norm_bias: Tensor,
    up: QLinear,
    down: QLinear,
    ff_norm_weight: Tensor,
    ff_norm_bias: Tensor,
    eps: f64,
}

impl Block {
    /// Load block `i`.
    pub fn load(weights: &mut Weights, config: &BertConfig, i: usize) -> Result<Self> {
        let name = |part: &str| format!("blk.{i}.{part}");
        let linear = |w: &mut Weights, prefix: String| -> Result<QLinear> {
            let weight = w.quantized(&format!("{prefix}.weight"))?;
            let bias = w.optional(&format!("{prefix}.bias"));
            Ok(QLinear::new(weight, bias)?)
        };

        // The header's feed-forward width and the tensor's must agree. They
        // are written independently, and a disagreement means one of the two
        // is describing a different model — which would otherwise surface as
        // a shape error several layers deeper, pointing at the wrong place.
        let up = linear(weights, name("ffn_up"))?;
        match up.output_width() {
            Some(found) if found != config.ff => {
                return Err(crate::error::Error::MissingFromGguf {
                    what: format!(
                        "blk.{i}.ffn_up sized for the declared feed-forward width {}; \
                         the tensor is {found} wide",
                        config.ff
                    ),
                });
            }
            // A bias-less projection reveals no width, so there is nothing to
            // check — the matmul will still catch a genuine mismatch, just
            // with a less specific message.
            _ => {}
        }

        Ok(Self {
            attention: Attention::load(weights, config, i)?,
            attn_norm_weight: weights.dequantized(&name("attn_output_norm.weight"))?,
            attn_norm_bias: weights.dequantized(&name("attn_output_norm.bias"))?,
            up,
            down: linear(weights, name("ffn_down"))?,
            ff_norm_weight: weights.dequantized(&name("layer_output_norm.weight"))?,
            ff_norm_bias: weights.dequantized(&name("layer_output_norm.bias"))?,
            eps: config.eps,
        })
    }

    /// Run `xs` through the block under `mask`.
    pub fn forward(&self, xs: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let attended = self.attention.forward(xs, mask)?;
        let xs = layer_norm(
            &(attended + xs)?,
            &self.attn_norm_weight,
            &self.attn_norm_bias,
            self.eps,
        )?;

        let ff = self.down.forward(&gelu(&self.up.forward(&xs)?)?)?;
        Ok(layer_norm(
            &(ff + &xs)?,
            &self.ff_norm_weight,
            &self.ff_norm_bias,
            self.eps,
        )?)
    }
}
