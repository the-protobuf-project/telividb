//! Rotary position embeddings.
//!
//! `nomic-bert` has no learned position table: position enters through a
//! rotation applied to queries and keys inside every attention block. That is
//! why [`Embeddings`] finds no `position_embd.weight` in a nomic GGUF — the
//! tensor is genuinely absent, not missing.
//!
//! [`Embeddings`]: super::embeddings::Embeddings

use crate::error::Result;
use candle_core::{D, Device, Tensor};

/// Precomputed cosine and sine tables, shared by every layer.
///
/// Built once per model rather than per block: the frequencies depend only on
/// the head width and the base, so all twelve blocks would otherwise compute
/// and hold twelve identical copies.
pub struct Rope {
    cos: Tensor,
    sin: Tensor,
}

impl Rope {
    /// Build tables covering `max_seq` positions for `dim`-wide heads.
    pub fn new(dim: usize, max_seq: usize, base: f64, device: &Device) -> Result<Self> {
        let half = dim / 2;
        let inv_freq: Vec<f32> = (0..half)
            .map(|i| 1f32 / (base as f32).powf(2.0 * i as f32 / dim as f32))
            .collect();

        let inv_freq = Tensor::new(inv_freq.as_slice(), device)?.unsqueeze(0)?;
        let positions = Tensor::arange(0u32, max_seq as u32, device)?
            .to_dtype(candle_core::DType::F32)?
            .reshape((max_seq, 1))?;
        let freqs = positions.matmul(&inv_freq)?;

        Ok(Self {
            cos: freqs.cos()?,
            sin: freqs.sin()?,
        })
    }

    /// Rotate `x`, shaped `[batch, heads, seq, head_dim]`.
    ///
    /// The non-interleaved (GPT-NeoX) convention: the head is split in half
    /// and the halves are rotated against each other. nomic-bert is trained
    /// this way — the interleaved (GPT-J) convention pairs *adjacent* elements
    /// instead, and using the wrong one leaves every vector well-formed while
    /// scrambling the positional signal completely.
    pub fn apply(&self, x: &Tensor) -> Result<Tensor> {
        let (_, _, seq, dim) = x.dims4()?;
        let half = dim / 2;

        // Narrowed to this batch's length: the tables cover the model's full
        // context, which is far longer than a typical batch.
        let cos = self.cos.narrow(0, 0, seq)?.reshape((1, 1, seq, half))?;
        let sin = self.sin.narrow(0, 0, seq)?.reshape((1, 1, seq, half))?;

        let x1 = x.narrow(D::Minus1, 0, half)?;
        let x2 = x.narrow(D::Minus1, half, half)?;

        let rotated_1 = (x1.broadcast_mul(&cos)? - x2.broadcast_mul(&sin)?)?;
        let rotated_2 = (x1.broadcast_mul(&sin)? + x2.broadcast_mul(&cos)?)?;
        Ok(Tensor::cat(&[rotated_1, rotated_2], D::Minus1)?)
    }
}

#[cfg(test)]
#[path = "rope_test.rs"]
mod tests;
