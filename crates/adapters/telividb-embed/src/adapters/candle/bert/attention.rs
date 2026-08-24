//! Multi-head self-attention over a padded batch.

use super::linear::QLinear;
use super::ops::softmax;
use crate::adapters::candle::config::BertConfig;
use crate::adapters::candle::weights::Weights;
use crate::error::Result;
use candle_core::Tensor;

/// One block's attention: three projections in, one out.
pub struct Attention {
    query: QLinear,
    key: QLinear,
    value: QLinear,
    output: QLinear,
    heads: usize,
    head_dim: usize,
    /// `1/sqrt(head_dim)`, precomputed — it is constant per model and this is
    /// the innermost loop in the encoder.
    scale: f64,
}

impl Attention {
    /// Load block `i`'s attention weights.
    pub fn load(weights: &mut Weights, config: &BertConfig, i: usize) -> Result<Self> {
        let at = |part: &str| format!("blk.{i}.attn_{part}");
        Ok(Self {
            query: load(weights, &at("q"))?,
            key: load(weights, &at("k"))?,
            value: load(weights, &at("v"))?,
            output: load(weights, &at("output"))?,
            heads: config.heads,
            head_dim: config.head_dim(),
            scale: 1.0 / (config.head_dim() as f64).sqrt(),
        })
    }

    /// Attend over `xs` (`[batch, seq, hidden]`), with `mask` additive and
    /// broadcastable over heads.
    pub fn forward(&self, xs: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let (batch, seq, _) = xs.dims3()?;
        let split = |t: Tensor| -> candle_core::Result<Tensor> {
            t.reshape((batch, seq, self.heads, self.head_dim))?
                .transpose(1, 2)?
                .contiguous()
        };

        let q = split(self.query.forward(xs)?)?;
        let k = split(self.key.forward(xs)?)?;
        let v = split(self.value.forward(xs)?)?;

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

/// Load a `<prefix>.weight` / `<prefix>.bias` pair.
fn load(weights: &mut Weights, prefix: &str) -> Result<QLinear> {
    let w = weights.quantized(&format!("{prefix}.weight"))?;
    let b = weights.optional(&format!("{prefix}.bias"));
    Ok(QLinear::new(w, b)?)
}
