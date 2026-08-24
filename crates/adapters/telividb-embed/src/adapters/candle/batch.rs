//! Turning texts into a padded tensor pair.

use crate::domain::Task;
use crate::error::{Error, Result};
use candle_core::{DType, Device, Tensor};
use tokenizers::Tokenizer;

/// Token ids and their attention mask, both `[batch, seq]`.
pub struct Batch {
    /// Token ids, padded to the batch's longest sequence.
    pub ids: Tensor,
    /// 1 for a real token, 0 for padding.
    pub attention: Tensor,
}

/// Tokenize `texts` and pad them into one batch.
///
/// Padded to the longest sequence *in this batch* rather than to the model's
/// context: a batch of short texts padded to 8192 would do three orders of
/// magnitude more attention work than it needs, and attention is quadratic in
/// sequence length.
pub fn encode(
    tokenizer: &Tokenizer,
    texts: &[String],
    task: Task,
    context: usize,
    device: &Device,
) -> Result<Batch> {
    let prefixed: Vec<String> = texts.iter().map(|t| prefix(task, t)).collect();
    let encoded = tokenizer
        .encode_batch(prefixed, true)
        .map_err(|e| Error::Tokenizer(e.to_string()))?;

    // Truncated rather than refused: the position embeddings simply do not
    // reach past `context`, and an out-of-range index is a hard tensor error
    // where a truncated tail is a degraded but usable vector.
    let width = encoded
        .iter()
        .map(|e| e.get_ids().len())
        .max()
        .unwrap_or(0)
        .min(context)
        .max(1);

    let mut ids = Vec::with_capacity(encoded.len() * width);
    let mut attention = Vec::with_capacity(encoded.len() * width);
    for item in &encoded {
        let row = &item.get_ids()[..item.get_ids().len().min(width)];
        ids.extend_from_slice(row);
        ids.extend(std::iter::repeat_n(0u32, width - row.len()));
        attention.extend(std::iter::repeat_n(1u32, row.len()));
        attention.extend(std::iter::repeat_n(0u32, width - row.len()));
    }

    let shape = (encoded.len(), width);
    Ok(Batch {
        ids: Tensor::from_vec(ids, shape, device)?.to_dtype(DType::U32)?,
        attention: Tensor::from_vec(attention, shape, device)?.to_dtype(DType::U32)?,
    })
}

/// Apply the model's task prefix.
///
/// nomic-embed and e5 were trained with these, and the asymmetry is the point:
/// the same sentence stored and searched produces deliberately different
/// vectors. Omitting the prefix lowers recall measurably while returning
/// perfectly well-formed vectors, so nothing surfaces the mistake.
fn prefix(task: Task, text: &str) -> String {
    match task {
        Task::Document => format!("search_document: {text}"),
        Task::Query => format!("search_query: {text}"),
    }
}

#[cfg(test)]
#[path = "batch_test.rs"]
mod tests;
