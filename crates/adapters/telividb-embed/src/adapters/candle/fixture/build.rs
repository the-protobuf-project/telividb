//! Assembling the fixture GGUF.

use super::tensors::{filled, quantized, vector};
use candle_core::Device;
use candle_core::quantized::QTensor;
use candle_core::quantized::gguf_file::{Value, write};
use std::path::Path;

/// The fixture's shape, so tests can assert against it without restating
/// magic numbers.
pub struct TinyModel {
    /// How many blocks to write. Two, so a bug in the residual chain
    /// between layers is reachable — one layer would never exercise it.
    pub layers: usize,
    /// Hidden width, and so the embedding width the model produces.
    pub hidden: usize,
    /// Feed-forward intermediate width.
    pub ff: usize,
    /// Heads to split the hidden width across. Two rather than one, so a
    /// head-reshaping mistake shows up instead of being the identity.
    pub heads: usize,
    /// How many tokens the vocabulary holds. Must cover the fixed special
    /// tokens plus the test corpus, or the ids the metadata points at would
    /// fall outside it.
    pub vocab: usize,
    /// Longest sequence the position embeddings cover.
    pub context: usize,
}

impl Default for TinyModel {
    fn default() -> Self {
        Self {
            layers: 2,
            hidden: 8,
            ff: 16,
            heads: 2,
            vocab: 24,
            context: 16,
        }
    }
}

/// Write a loadable GGUF encoder to `path`.
pub fn write_tiny_gguf(path: &Path, model: &TinyModel) -> candle_core::Result<()> {
    let device = Device::Cpu;
    let vocab: Vec<Value> = tiny_vocab(model.vocab)
        .into_iter()
        .map(Value::String)
        .collect();

    let metadata: Vec<(&str, Value)> = vec![
        ("general.architecture", Value::String("bert".to_owned())),
        ("bert.block_count", Value::U32(model.layers as u32)),
        ("bert.embedding_length", Value::U32(model.hidden as u32)),
        ("bert.feed_forward_length", Value::U32(model.ff as u32)),
        ("bert.attention.head_count", Value::U32(model.heads as u32)),
        ("bert.attention.layer_norm_epsilon", Value::F32(1e-12)),
        ("bert.context_length", Value::U32(model.context as u32)),
        ("tokenizer.ggml.tokens", Value::Array(vocab)),
        ("tokenizer.ggml.unknown_token_id", Value::U32(1)),
        ("tokenizer.ggml.bos_token_id", Value::U32(2)),
        ("tokenizer.ggml.eos_token_id", Value::U32(3)),
    ];

    let tensors = build_tensors(model, &device)?;
    let borrowed: Vec<(&str, &Value)> = metadata.iter().map(|(k, v)| (*k, v)).collect();
    let tensor_refs: Vec<(&str, &QTensor)> = tensors.iter().map(|(k, t)| (k.as_str(), t)).collect();

    let mut file = std::fs::File::create(path)?;
    write(&mut file, &borrowed, &tensor_refs)
}

/// Every tensor the encoder looks for, at the fixture's dimensions.
fn build_tensors(
    model: &TinyModel,
    device: &Device,
) -> candle_core::Result<Vec<(String, QTensor)>> {
    let h = model.hidden;
    let mut out = vec![
        (
            "token_embd.weight".to_owned(),
            quantized(&filled((model.vocab, h), 0.1, device)?)?,
        ),
        (
            "position_embd.weight".to_owned(),
            quantized(&filled((model.context, h), 0.2, device)?)?,
        ),
        (
            "token_embd_norm.weight".to_owned(),
            quantized(&vector(h, 0.3, device)?)?,
        ),
        (
            "token_embd_norm.bias".to_owned(),
            quantized(&vector(h, 0.4, device)?)?,
        ),
    ];

    for i in 0..model.layers {
        let seed = i as f32;
        for part in ["q", "k", "v", "output"] {
            out.push((
                format!("blk.{i}.attn_{part}.weight"),
                quantized(&filled((h, h), seed + 1.0, device)?)?,
            ));
            out.push((
                format!("blk.{i}.attn_{part}.bias"),
                quantized(&vector(h, seed + 2.0, device)?)?,
            ));
        }
        // Note the orientation: a GGUF weight is stored `[out, in]`, and
        // `QMatMul` multiplies against its transpose. Writing `ffn_up` as
        // `[hidden, ff]` would load without error and fail at the matmul.
        out.push((
            format!("blk.{i}.ffn_up.weight"),
            quantized(&filled((model.ff, h), seed + 3.0, device)?)?,
        ));
        out.push((
            format!("blk.{i}.ffn_up.bias"),
            quantized(&vector(model.ff, seed + 4.0, device)?)?,
        ));
        out.push((
            format!("blk.{i}.ffn_down.weight"),
            quantized(&filled((h, model.ff), seed + 5.0, device)?)?,
        ));
        out.push((
            format!("blk.{i}.ffn_down.bias"),
            quantized(&vector(h, seed + 6.0, device)?)?,
        ));
        for part in ["attn_output_norm", "layer_output_norm"] {
            out.push((
                format!("blk.{i}.{part}.weight"),
                quantized(&vector(h, seed + 7.0, device)?)?,
            ));
            out.push((
                format!("blk.{i}.{part}.bias"),
                quantized(&vector(h, seed + 8.0, device)?)?,
            ));
        }
    }
    Ok(out)
}

/// A WordPiece vocabulary just large enough to tokenize the test corpus.
///
/// Order is the token id, and the first four positions are fixed because the
/// metadata above points at them by index.
fn tiny_vocab(size: usize) -> Vec<String> {
    let mut tokens = vec![
        "[PAD]".to_owned(),
        "[UNK]".to_owned(),
        "[CLS]".to_owned(),
        "[SEP]".to_owned(),
    ];
    for word in [
        "search", "document", "query", ":", "the", "cat", "sat", "dog",
    ] {
        tokens.push(word.to_owned());
    }
    while tokens.len() < size {
        tokens.push(format!("tok{}", tokens.len()));
    }
    tokens.truncate(size);
    tokens
}
