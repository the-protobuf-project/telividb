//! Multi-head self-attention over a padded batch.

use super::linear::QLinear;
use super::ops::softmax;
use super::qkv::Qkv;
use super::rope::Rope;
use crate::adapters::candle::config::BertConfig;
use crate::adapters::candle::weights::Weights;
use crate::error::Result;
use candle_core::Tensor;

/// One block's attention: a projection in, a projection out.
pub struct Attention {
    qkv: Qkv,
    output: QLinear,
    hidden: usize,
    heads: usize,
    head_dim: usize,
    /// `1/sqrt(head_dim)`, precomputed — constant per model, and this is the
    /// innermost loop in the encoder.
    scale: f64,
}

impl Attention {
    /// Load block `i`'s attention weights.
    pub fn load(weights: &mut Weights, config: &BertConfig, i: usize) -> Result<Self> {
        let weight = weights.quantized(&format!("blk.{i}.attn_output.weight"))?;
        let bias = weights.optional(&format!("blk.{i}.attn_output.bias"));
        Ok(Self {
            qkv: Qkv::load(weights, i)?,
            output: QLinear::new(weight, bias)?,
            hidden: config.hidden,
            heads: config.heads,
            head_dim: config.head_dim(),
            scale: 1.0 / (config.head_dim() as f64).sqrt(),
        })
    }

    /// Attend over `xs` (`[batch, seq, hidden]`), with `mask` additive and
    /// broadcastable over heads.
    ///
    /// `rope` is applied to queries and keys when the model carries no learned
    /// position table — that rotation is the only thing telling the model
    /// where a token sits.
    pub fn forward(&self, xs: &Tensor, mask: &Tensor, rope: Option<&Rope>) -> Result<Tensor> {
        let (batch, seq, _) = xs.dims3()?;
        let split = |t: Tensor| -> candle_core::Result<Tensor> {
            t.reshape((batch, seq, self.heads, self.head_dim))?
                .transpose(1, 2)?
                .contiguous()
        };

        let (q, k, v) = self.qkv.project(xs, self.hidden)?;
        let (mut q, mut k, v) = (split(q)?, split(k)?, split(v)?);
        if let Some(rope) = rope {
            q = rope.apply(&q)?;
            k = rope.apply(&k)?;
        }

        // The mask is added, not multiplied: a large negative before the
        // softmax sends a padded position to ~0 weight. Zeroing afterwards
        // instead would leave the row un-normalized, quietly scaling every
        // real token by however much weight the padding took.
        let scores = (q.matmul(&k.transpose(2, 3)?)? * self.scale)?;
        let weights = softmax(&scores.broadcast_add(mask)?)?;

        let context = weights.matmul(&v)?.transpose(1, 2)?.reshape((
            batch,
            seq,
            self.heads * self.head_dim,
        ))?;
        Ok(self.output.forward(&context)?)
    }
}
