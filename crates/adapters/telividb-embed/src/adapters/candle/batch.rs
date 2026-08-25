//! Turning texts into padded tensor pairs.
//!
//! Tokenizing and tensor-building are separate steps because the scheduler
//! between them needs to know how long each text is before deciding what to
//! batch with what — see [`super::schedule`]. Tokenizing inside the tensor
//! step would mean doing it twice, or batching blind.

use crate::domain::Task;
use crate::error::{Error, Result};
use candle_core::{DType, Device, Tensor};
use tokenizers::Tokenizer;

/// Token ids and their attention mask, both `[rows, seq]`.
pub struct Batch {
    /// Token ids, padded to the batch's longest sequence.
    pub ids: Tensor,
    /// 1 for a real token, 0 for padding.
    pub attention: Tensor,
}

/// Tokenize every text, truncated to what the model's positions cover.
///
/// Truncated rather than refused: the position embeddings simply do not reach
/// past `context`, and an out-of-range index is a hard tensor error where a
/// truncated tail is a degraded but usable vector.
pub fn tokenize(
    tokenizer: &Tokenizer,
    texts: &[String],
    task: Task,
    context: usize,
) -> Result<Vec<Vec<u32>>> {
    let prefixed: Vec<String> = texts.iter().map(|t| prefix(task, t)).collect();
    let encoded = tokenizer
        .encode_batch(prefixed, true)
        .map_err(|e| Error::Tokenizer(e.to_string()))?;

    Ok(encoded
        .iter()
        .map(|item| {
            let ids = item.get_ids();
            ids[..ids.len().min(context)].to_vec()
        })
        .collect())
}

/// Build the tensors for one batch of already-tokenized rows.
///
/// Padded to the longest row *in this batch*, which is what the scheduler
/// works to keep small.
pub fn to_tensors(rows: &[&[u32]], device: &Device) -> Result<Batch> {
    // At least one column: a zero-width tensor is a shape error several layers
    // deeper, pointing at the wrong place.
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0).max(1);

    let mut ids = Vec::with_capacity(rows.len() * width);
    let mut attention = Vec::with_capacity(rows.len() * width);
    for row in rows {
        ids.extend_from_slice(row);
        ids.extend(std::iter::repeat_n(0u32, width - row.len()));
        attention.extend(std::iter::repeat_n(1u32, row.len()));
        attention.extend(std::iter::repeat_n(0u32, width - row.len()));
    }

    let shape = (rows.len(), width);
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
