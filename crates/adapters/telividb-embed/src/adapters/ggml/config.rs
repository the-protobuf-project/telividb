//! The architecture parameters, read out of the GGUF header.

use crate::error::Result;
use telividb_compute::{Header, Weights};

/// Architectures the encoder implements a forward pass for.
///
/// Defined by [`Architecture`](telividb_core::Architecture) rather than
/// repeated here. The catalog refuses to *download* what this refuses to load,
/// and two lists would eventually disagree — with the symptom being a gigabyte
/// fetched before the refusal.
pub const SUPPORTED: &[&str] = telividb_core::Architecture::NAMES;

/// Which forward pass a model needs.
///
/// Two families, and the split is not cosmetic: they differ in where the norms
/// sit, which norm it is, and whether keys are shared across heads. Running one
/// family's pass over the other's weights produces finite, correctly shaped,
/// wrong vectors — so the family is decided once, from the header, rather than
/// inferred per block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// BERT and nomic-bert: post-norm, LayerNorm with bias, one key per head.
    Bert,
    /// Qwen3, Llama and their relatives: pre-norm, RMSNorm without bias,
    /// grouped-query attention, and a gated feed-forward network.
    Causal,
}

/// One model's shape.
#[derive(Debug, Clone)]
pub struct Config {
    /// Metadata key prefix, which is the architecture name.
    pub arch: String,
    /// Which forward pass this model needs.
    pub family: Family,
    /// How many transformer blocks the forward pass runs.
    ///
    /// Every declared block must exist as tensors; a count larger than the file
    /// carries fails at load rather than mid-encode.
    pub layers: usize,
    /// Width of every activation, and of the vector this model produces.
    pub hidden: usize,
    /// Query heads, each attending over its own score matrix.
    ///
    /// Not necessarily `hidden / head_dim`: several models widen their heads
    /// past the residual stream, so the two are read separately.
    pub heads: usize,
    /// Key and value heads, which may be fewer than [`heads`](Self::heads).
    ///
    /// Grouped-query attention shares one key head across several query heads —
    /// eight-to-two is typical. Equal to `heads` for a model without it, so the
    /// forward pass reads this rather than branching on whether it applies.
    pub kv_heads: usize,
    /// Width of one attention head.
    ///
    /// Read from the header rather than derived, because `hidden / heads` is
    /// **wrong** for several current models: Qwen3-Embedding-0.6B is 1024 wide
    /// with 16 heads of 128, so the heads are deliberately wider than the model.
    /// Deriving it there yields 64 and every reshape downstream is misaligned.
    pub head_dim: usize,
    /// Normalization epsilon, as the model was trained with it.
    pub eps: f32,
    /// Longest sequence the position scheme covers.
    pub context: usize,
    /// Rotary frequency base, present only for models using RoPE.
    ///
    /// `None` means the model carries a learned position table instead, and the
    /// two are not interchangeable — applying neither leaves the encoder with
    /// no positional signal at all, which reads as a subtle quality loss rather
    /// than a failure.
    pub rope_base: Option<f32>,
}

impl Config {
    /// Read the shape from a loaded model's header.
    pub fn from_weights(weights: &Weights) -> Result<Self> {
        Self::from_header(weights.header())
    }

    /// Width of one attention head.
    pub fn head_dim(&self) -> usize {
        self.head_dim
    }

    /// How many query heads share each key head.
    ///
    /// One for ordinary multi-head attention. Above one, keys and values are
    /// tiled up to the query head count before the scores matmul.
    pub fn heads_per_kv(&self) -> usize {
        self.heads / self.kv_heads.max(1)
    }

    /// `1/sqrt(head_dim)` — the attention scale.
    ///
    /// Over the **head** width, not the model width. Scaling by the wrong one
    /// pushes softmax into saturation, where a single logit dominates and the
    /// head stops mixing.
    pub fn scale(&self) -> f32 {
        1.0 / (self.head_dim() as f32).sqrt()
    }

    /// Whether position enters through rotation rather than a learned table.
    pub fn uses_rope(&self) -> bool {
        self.rope_base.is_some()
    }

    /// Read a required `u32`, naming the key when it is absent.
    pub(super) fn u32(weights: &Header, key: &str) -> Result<u32> {
        weights
            .u32_meta(key)
            .ok_or_else(|| crate::error::Error::MissingFromGguf {
                what: key.to_owned(),
            })
    }
}
