//! Encoder hyperparameters, read from the GGUF rather than assumed.

use crate::error::{Error, Result};
use candle_core::quantized::gguf_file::Content;

/// Architectures this encoder implements.
///
/// Both are BERT: `nomic-bert` differs from `bert` in its training recipe and
/// its rotary-free long context, not in the forward pass this file describes.
/// Anything else is refused (see [`Error::UnsupportedArchitecture`]) rather
/// than run hopefully — a mismatched architecture finds the tensors it expects
/// by name, runs to completion, and returns wrong vectors.
pub const SUPPORTED: &[&str] = &["bert", "nomic-bert"];

/// Everything the forward pass needs to know about the model's shape.
#[derive(Debug, Clone)]
pub struct BertConfig {
    /// Metadata key prefix, which is the architecture name (`bert.` etc.).
    pub arch: String,
    /// Number of transformer blocks.
    pub layers: usize,
    /// Hidden width, and therefore the embedding width this model produces.
    pub hidden: usize,
    /// Width of the feed-forward intermediate projection.
    pub ff: usize,
    /// Attention heads. `hidden` must divide by this evenly.
    pub heads: usize,
    /// Epsilon inside every layer norm.
    pub eps: f64,
    /// Longest sequence the position embeddings cover.
    pub context: usize,
}

impl BertConfig {
    /// Read the shape from `content`'s metadata.
    pub fn from_gguf(content: &Content) -> Result<Self> {
        let arch = string(content, "general.architecture")?;
        if !SUPPORTED.contains(&arch.as_str()) {
            return Err(Error::UnsupportedArchitecture {
                found: arch,
                supported: SUPPORTED,
            });
        }

        let hidden = usize_at(content, &format!("{arch}.embedding_length"))?;
        let heads = usize_at(content, &format!("{arch}.attention.head_count"))?;
        if heads == 0 || hidden % heads != 0 {
            return Err(Error::MissingFromGguf {
                what: format!(
                    "a head count dividing {hidden} evenly; the file says {heads}, \
                     which cannot describe this model"
                ),
            });
        }

        Ok(Self {
            layers: usize_at(content, &format!("{arch}.block_count"))?,
            hidden,
            ff: usize_at(content, &format!("{arch}.feed_forward_length"))?,
            heads,
            eps: f64_at(content, &format!("{arch}.attention.layer_norm_epsilon"))?,
            context: usize_at(content, &format!("{arch}.context_length"))?,
            arch,
        })
    }

    /// Width of one attention head.
    pub fn head_dim(&self) -> usize {
        self.hidden / self.heads
    }
}

/// Read a string-valued metadata key.
fn string(content: &Content, key: &str) -> Result<String> {
    let value = content.metadata.get(key).ok_or_else(missing(key))?;
    value
        .to_string()
        .map(|s| s.to_owned())
        .map_err(|_| missing(key)())
}

/// Read an integer-valued metadata key, whatever width it was written at.
///
/// GGUF writers are not consistent about integer width for the same key, so
/// matching on one type would reject files that are perfectly valid.
fn usize_at(content: &Content, key: &str) -> Result<usize> {
    let value = content.metadata.get(key).ok_or_else(missing(key))?;
    value
        .to_u32()
        .map(|v| v as usize)
        .or_else(|_| value.to_u64().map(|v| v as usize))
        .or_else(|_| value.to_i32().map(|v| v as usize))
        .map_err(|_| missing(key)())
}

/// Read a float-valued metadata key at either width.
fn f64_at(content: &Content, key: &str) -> Result<f64> {
    let value = content.metadata.get(key).ok_or_else(missing(key))?;
    value
        .to_f32()
        .map(|v| v as f64)
        .or_else(|_| value.to_f64())
        .map_err(|_| missing(key)())
}

/// The "absent or unreadable" error for `key`, as a closure both paths reuse.
fn missing(key: &str) -> impl Fn() -> Error + '_ {
    move || Error::MissingFromGguf {
        what: key.to_owned(),
    }
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
