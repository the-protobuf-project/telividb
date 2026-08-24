use super::*;
use crate::adapters::candle::fixture::{TinyModel, write_tiny_gguf};
use crate::adapters::candle::tokenize;
use candle_core::quantized::gguf_file::Content;

fn tokenizer() -> Tokenizer {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();
    let mut file = std::fs::File::open(&path).unwrap();
    tokenize::from_gguf(&Content::read(&mut file).unwrap()).unwrap()
}

#[test]
fn a_batch_is_padded_to_its_own_longest_row_not_to_the_context() {
    // Attention is quadratic in sequence length, so padding a batch of short
    // texts to the model's full context does orders of magnitude more work
    // than it needs to.
    let texts = vec!["the cat".to_owned(), "the dog sat".to_owned()];
    let batch = encode(&tokenizer(), &texts, Task::Document, 4096, &Device::Cpu).unwrap();

    let (rows, width) = batch.ids.dims2().unwrap();
    assert_eq!(rows, 2);
    assert!(
        width < 64,
        "padded to {width}, which is not this batch's own length"
    );
    assert_eq!(batch.attention.dims2().unwrap(), (rows, width));
}

#[test]
fn padding_is_masked_out_and_real_tokens_are_kept() {
    let texts = vec!["the cat sat".to_owned(), "the".to_owned()];
    let batch = encode(&tokenizer(), &texts, Task::Document, 4096, &Device::Cpu).unwrap();
    let mask = batch.attention.to_vec2::<u32>().unwrap();

    let first: u32 = mask[0].iter().sum();
    let second: u32 = mask[1].iter().sum();
    assert!(
        first > second,
        "the longer text should have more unmasked tokens: {mask:?}"
    );
    // Whatever the split, a mask entry is only ever 0 or 1.
    assert!(mask.iter().flatten().all(|v| *v <= 1));
}

#[test]
fn a_text_longer_than_the_context_is_truncated_rather_than_erroring() {
    // The position embeddings simply do not reach past the context. An
    // out-of-range index is a hard tensor failure; a truncated tail is a
    // degraded but usable vector.
    let long = vec!["the cat sat ".repeat(200)];
    let batch = encode(&tokenizer(), &long, Task::Document, 16, &Device::Cpu).unwrap();

    let (_, width) = batch.ids.dims2().unwrap();
    assert_eq!(width, 16);
}

#[test]
fn document_and_query_get_different_prefixes() {
    // The asymmetry is trained in. Dropping it lowers recall while returning
    // perfectly well-formed vectors, so nothing else would surface it.
    assert_ne!(prefix(Task::Document, "x"), prefix(Task::Query, "x"));
    assert!(prefix(Task::Document, "x").starts_with("search_document:"));
    assert!(prefix(Task::Query, "x").starts_with("search_query:"));
}

#[test]
fn an_empty_batch_still_produces_a_usable_width() {
    // A zero-width tensor is a shape error several layers deeper, pointing at
    // the wrong place.
    let batch = encode(
        &tokenizer(),
        &["".to_owned()],
        Task::Query,
        32,
        &Device::Cpu,
    )
    .unwrap();
    let (_, width) = batch.ids.dims2().unwrap();
    assert!(width >= 1);
}
