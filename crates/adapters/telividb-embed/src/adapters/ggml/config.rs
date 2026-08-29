//! The architecture parameters, read out of the GGUF header.
//!
//! **Every value here is read, never defaulted.** Layer count, head count,
//! epsilon and rotary base all vary between models of the same family, and a
//! wrong one produces finite, correctly-shaped, wrong vectors — the failure
//! mode with no symptom. A missing key is an error at load time, which is the
//! only place it can still be caught cheaply.

use crate::error::{Error, Result};
use telividb_compute::{Header, Weights};

/// Architectures whose tensor layout this encoder understands.
///
/// Checked rather than assumed: a mismatched architecture finds some of the
/// tensors it expects and silently misreads the rest.
/// Defined by [`Architecture`] rather than repeated here. The catalog refuses
/// to *download* what this refuses to load, and two lists would eventually
/// disagree — with the symptom being a gigabyte fetched before the refusal.
pub const SUPPORTED: &[&str] = telividb_core::Architecture::NAMES;

/// One model's shape.
#[derive(Debug, Clone)]
pub struct Config {
    /// Metadata key prefix, which is the architecture name.
    pub arch: String,
    /// How many transformer blocks the forward pass runs.
    ///
    /// Every declared block must exist as tensors; a count larger than the file
    /// carries fails at load rather than mid-encode.
    pub layers: usize,
    /// Width of every activation, and of the vector this model produces.
    pub hidden: usize,
    /// Attention heads; `hidden` must divide by this.
    pub heads: usize,
    /// Layer-norm epsilon, as the model was trained with it.
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

    /// The same, from a header parsed on its own.
    pub fn from_header(weights: &Header) -> Result<Self> {
        let arch =
            weights
                .str_meta("general.architecture")
                .ok_or_else(|| Error::MissingFromGguf {
                    what: "general.architecture".to_owned(),
                })?;
        if !SUPPORTED.contains(&arch.as_str()) {
            return Err(Error::UnsupportedArchitecture {
                found: arch,
                supported: SUPPORTED,
            });
        }

        let hidden = Self::u32(weights, &format!("{arch}.embedding_length"))? as usize;
        let heads = Self::u32(weights, &format!("{arch}.attention.head_count"))? as usize;
        if heads == 0 || !hidden.is_multiple_of(heads) {
            return Err(Error::MissingFromGguf {
                what: format!("a head count that divides {hidden}; found {heads}"),
            });
        }

        Ok(Self {
            layers: Self::u32(weights, &format!("{arch}.block_count"))? as usize,
            hidden,
            heads,
            eps: weights
                .f32_meta(&format!("{arch}.attention.layer_norm_epsilon"))
                .ok_or_else(|| Error::MissingFromGguf {
                    what: format!("{arch}.attention.layer_norm_epsilon"),
                })?,
            context: Self::u32(weights, &format!("{arch}.context_length"))? as usize,
            rope_base: weights.f32_meta(&format!("{arch}.rope.freq_base")),
            arch,
        })
    }

    /// Width of one attention head.
    pub fn head_dim(&self) -> usize {
        self.hidden / self.heads
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
    fn u32(weights: &Header, key: &str) -> Result<u32> {
        weights.u32_meta(key).ok_or_else(|| Error::MissingFromGguf {
            what: key.to_owned(),
        })
    }
}
