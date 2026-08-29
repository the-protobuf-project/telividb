//! Reading a [`Config`] out of a GGUF header.
//!
//! Split from `config.rs` because the two change for different reasons: that
//! file is the shape the forward pass reads, and this is the archaeology of
//! which key each publisher happened to write it under.

use super::config::{Config, Family, SUPPORTED};
use crate::error::{Error, Result};
use telividb_compute::Header;

impl Config {
    /// Read the shape from a header parsed on its own.
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
        let family = match arch.as_str() {
            "bert" | "nomic-bert" => Family::Bert,
            _ => Family::Causal,
        };

        let hidden = Self::u32(weights, &format!("{arch}.embedding_length"))? as usize;
        let heads = Self::u32(weights, &format!("{arch}.attention.head_count"))? as usize;
        if heads == 0 {
            return Err(Error::MissingFromGguf {
                what: "a non-zero attention.head_count".to_owned(),
            });
        }

        let head_dim = head_dim(weights, &arch, hidden, heads)?;
        let kv_heads = weights
            .u32_meta(&format!("{arch}.attention.head_count_kv"))
            .map_or(heads, |n| n as usize)
            .max(1);
        if !heads.is_multiple_of(kv_heads) {
            return Err(Error::MissingFromGguf {
                what: format!(
                    "a key-head count dividing {heads}; found {kv_heads}. \
                     Grouped-query attention tiles keys up to the query heads, \
                     which only works at a whole ratio"
                ),
            });
        }

        Ok(Self {
            family,
            layers: Self::u32(weights, &format!("{arch}.block_count"))? as usize,
            hidden,
            heads,
            kv_heads,
            head_dim,
            eps: eps(weights, &arch)?,
            context: Self::u32(weights, &format!("{arch}.context_length"))? as usize,
            rope_base: weights.f32_meta(&format!("{arch}.rope.freq_base")),
            arch,
        })
    }
}

/// The width of one attention head.
///
/// Prefers the header's own `attention.key_length`, because `hidden / heads` is
/// wrong wherever a model widens its heads past its residual stream —
/// Qwen3-Embedding-0.6B being the case that forced this. Falls back to the
/// division only when the key is absent, which is where BERT lives.
fn head_dim(weights: &Header, arch: &str, hidden: usize, heads: usize) -> Result<usize> {
    if let Some(width) = weights.u32_meta(&format!("{arch}.attention.key_length")) {
        return Ok(width as usize);
    }
    if !hidden.is_multiple_of(heads) {
        return Err(Error::MissingFromGguf {
            what: format!(
                "either {arch}.attention.key_length, or a head count dividing \
                 {hidden}; found {heads} heads and no key length"
            ),
        });
    }
    Ok(hidden / heads)
}

/// The normalization epsilon, under whichever key this family writes.
///
/// Two keys for one quantity: BERT writes `layer_norm_epsilon`, and every
/// RMS-normalized architecture writes `layer_norm_rms_epsilon`. Requiring only
/// the first is what made this crate reject Qwen3 files that were otherwise
/// complete.
fn eps(weights: &Header, arch: &str) -> Result<f32> {
    weights
        .f32_meta(&format!("{arch}.attention.layer_norm_epsilon"))
        .or_else(|| weights.f32_meta(&format!("{arch}.attention.layer_norm_rms_epsilon")))
        .ok_or_else(|| Error::MissingFromGguf {
            what: format!("{arch}.attention.layer_norm_epsilon or .layer_norm_rms_epsilon"),
        })
}
