use super::*;
use crate::adapters::ggml::vocab;

/// The downloaded model, if present.
///
/// Against the real vocabulary rather than a synthetic one, because what these
/// tests exercise — the `▁`/`##` convention, where the specials sit, whether
/// the post-processor terminates a sequence — is exactly what a hand-made
/// fixture would get right by construction and a real converter output might
/// not. Reading it costs a header parse, not a model load.
fn tokenizer() -> Option<Tokenizer> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/models/gguf/text/nomic-embed-text-v1.5.Q4_K_M.gguf");
    match path.exists() {
        true => Some(vocab::from_gguf(&path).unwrap()),
        false => {
            eprintln!("SKIPPED: run examples/models/download.sh to exercise this");
            None
        }
    }
}

#[test]
fn tokenizing_returns_one_row_per_text() {
    let texts = vec!["the cat".to_owned(), "the dog sat".to_owned()];
    let Some(tk) = tokenizer() else { return };
    let rows = tokenize(&tk, &texts, Task::Document, 4096).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| !r.is_empty()));
}

#[test]
fn a_text_longer_than_the_context_is_truncated_rather_than_erroring() {
    // The position embeddings do not reach past the context. An out-of-range
    // index is a hard tensor failure; a truncated tail is degraded but usable.
    let long = vec!["the cat sat ".repeat(200)];
    let Some(tk) = tokenizer() else { return };
    let rows = tokenize(&tk, &long, Task::Document, 16).unwrap();
    assert_eq!(rows[0].len(), 16);
}

#[test]
fn a_truncated_text_still_ends_with_the_terminator() {
    // Every encoding is wrapped `[CLS] ... [SEP]` by the post-processor, and
    // slicing from the right would drop the `[SEP]`. A sequence ending
    // mid-content is a shape the model never saw in training — nothing errors,
    // the vector is just drawn from a distribution the weights do not describe.
    let Some(tokenizer) = tokenizer() else { return };
    let short = tokenize(&tokenizer, &["the cat".to_owned()], Task::Document, 4096).unwrap();
    let terminator = *short[0].last().unwrap();
    let opener = short[0][0];

    let long = vec!["the cat sat ".repeat(200)];
    let truncated = tokenize(&tokenizer, &long, Task::Document, 16).unwrap();

    assert_eq!(
        truncated[0].last(),
        Some(&terminator),
        "a truncated sequence must keep the terminator the tokenizer emitted"
    );
    assert_eq!(
        truncated[0][0], opener,
        "and must still open with the same special token"
    );
}

#[test]
fn tensors_are_padded_to_the_batch_not_to_the_context() {
    // Attention is quadratic in the padded width, so padding a batch of short
    // rows out to the model's full context does orders of magnitude more work
    // than it needs.
    let rows: Vec<Vec<u32>> = vec![vec![4, 5, 6], vec![7]];
    let refs: Vec<&[u32]> = rows.iter().map(|r| r.as_slice()).collect();

    let batch = to_rows(&refs);
    // Two rows padded to the longest member's three, not to the model context.
    assert_eq!(batch.ids.len(), 2 * 3);
    assert_eq!(batch.ids, vec![4, 5, 6, 7, 0, 0]);
}

#[test]
fn padding_is_masked_out_and_real_tokens_are_kept() {
    let rows: Vec<Vec<u32>> = vec![vec![4, 5, 6], vec![7]];
    let refs: Vec<&[u32]> = rows.iter().map(|r| r.as_slice()).collect();

    let mask = to_rows(&refs).attention;
    assert_eq!(&mask[..3], &[1, 1, 1], "every real token kept");
    assert_eq!(&mask[3..], &[1, 0, 0], "the short row's padding masked");
}

#[test]
fn an_empty_row_still_produces_a_usable_width() {
    let refs: Vec<&[u32]> = vec![&[]];
    let batch = to_rows(&refs);
    let width = batch.ids.len();
    assert!(width >= 1);
}

#[test]
fn document_and_query_get_different_prefixes() {
    // The asymmetry is trained in. Dropping it lowers recall while returning
    // perfectly well-formed vectors, so nothing else would surface it.
    assert_ne!(prefix(Task::Document, "x"), prefix(Task::Query, "x"));
    assert!(prefix(Task::Document, "x").starts_with("search_document:"));
    assert!(prefix(Task::Query, "x").starts_with("search_query:"));
}
