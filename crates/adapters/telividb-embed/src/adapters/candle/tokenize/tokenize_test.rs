use super::*;
use crate::adapters::candle::fixture::{TinyModel, write_tiny_gguf};
use candle_core::quantized::gguf_file::Value;

fn content_of(path: &std::path::Path) -> Content {
    let mut file = std::fs::File::open(path).unwrap();
    Content::read(&mut file).unwrap()
}

fn fixture() -> (tempfile::TempDir, Content) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();
    let content = content_of(&path);
    (dir, content)
}

/// Replace the fixture's vocabulary with `tokens`.
fn with_vocab(content: &mut Content, tokens: &[&str]) {
    let values: Vec<Value> = tokens
        .iter()
        .map(|t| Value::String((*t).to_owned()))
        .collect();
    content
        .metadata
        .insert("tokenizer.ggml.tokens".to_owned(), Value::Array(values));
}

#[test]
fn a_tokenizer_is_built_from_the_gguf_alone() {
    // Rule 12 makes the file's digest the model's identity. A tokenizer read
    // from a separate file would sit outside that digest entirely.
    let (_dir, content) = fixture();
    let tokenizer = from_gguf(&content).unwrap();

    let encoded = tokenizer.encode("the cat", true).unwrap();
    assert!(!encoded.get_ids().is_empty());
}

#[test]
fn a_missing_vocabulary_is_reported_not_defaulted() {
    let (_dir, mut content) = fixture();
    content.metadata.remove("tokenizer.ggml.tokens");

    match from_gguf(&content) {
        Err(crate::Error::MissingFromGguf { what }) => {
            assert_eq!(what, "tokenizer.ggml.tokens");
        }
        other => panic!("expected a missing-vocabulary error, got {other:?}"),
    }
}

#[test]
fn a_token_containing_a_newline_is_refused_rather_than_shifting_every_id() {
    // The line-per-token encoding cannot represent one, and accepting it would
    // silently renumber the whole vocabulary from that point on.
    let (_dir, mut content) = fixture();
    let mut tokens: Vec<Value> = content.metadata["tokenizer.ggml.tokens"]
        .to_vec()
        .unwrap()
        .clone();
    tokens.push(Value::String("bad\ntoken".to_owned()));
    content
        .metadata
        .insert("tokenizer.ggml.tokens".to_owned(), Value::Array(tokens));

    assert!(matches!(
        from_gguf(&content),
        Err(crate::Error::Tokenizer(_))
    ));
}

#[test]
fn a_word_start_marker_is_converted_to_wordpiece_continuations() {
    // The bug this exists for: llama.cpp stores a BERT vocabulary in
    // SentencePiece convention — `▁` marks word starts, continuations go bare
    // — which is the inverse of WordPiece's `##`. Read as-is, every word
    // needing a subword split becomes `[UNK]` while short whole-word tokens
    // still resolve, so tokenization looks like it works.
    let (_dir, mut content) = fixture();
    with_vocab(
        &mut content,
        &["[PAD]", "[UNK]", "[CLS]", "[SEP]", "\u{2581}win", "dow"],
    );

    let tokenizer = from_gguf(&content).unwrap();
    let encoded = tokenizer.encode("window", false).unwrap();

    assert_eq!(
        encoded.get_tokens(),
        &["win".to_owned(), "##dow".to_owned()],
        "the word should split rather than fall back to [UNK]"
    );
    // Ids are positions in the array and must survive the rewrite untouched.
    assert_eq!(encoded.get_ids(), &[4, 5]);
}

#[test]
fn special_tokens_are_left_unprefixed_by_the_conversion() {
    // `##[CLS]` would be unmatchable, so the post-processor could never insert
    // it and every sequence would lose its markers.
    let tokens = ["[PAD]".to_owned(), "[CLS]".to_owned(), "cat".to_owned()];
    let converted = vocab::to_wordpiece(&tokens, None);
    assert_eq!(converted[0], "[PAD]");
    assert_eq!(converted[1], "[CLS]");
    assert_eq!(
        converted[2], "##cat",
        "a bare normal token is a continuation"
    );
}

#[test]
fn an_uncased_vocabulary_is_detected_despite_its_bracketed_specials() {
    // `[CLS]`, `[SEP]`, `[UNK]` are ASCII capitals in every BERT vocabulary.
    // Counting them concludes an entirely lowercase vocabulary is cased, and
    // then every capitalized word — the first of most sentences, and every
    // proper noun — becomes [UNK].
    let tokens: Vec<String> = ["[CLS]", "[SEP]", "[UNK]", "\u{2581}the", "\u{2581}cat"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert!(vocab::is_lowercase(&tokens, None));
}

#[test]
fn a_non_ascii_capital_does_not_make_a_vocabulary_look_cased() {
    // An uncased BERT vocabulary still holds a few of these — `ℝ` is in
    // nomic-embed-text-v1.5 — and a general `is_uppercase` test finds them.
    let tokens: Vec<String> = ["\u{2581}the", "\u{2581}\u{211d}", "\u{211d}"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert!(vocab::is_lowercase(&tokens, None));
}

#[test]
fn a_genuinely_cased_vocabulary_is_detected() {
    let tokens: Vec<String> = ["[CLS]", "\u{2581}The", "\u{2581}Rust"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert!(!vocab::is_lowercase(&tokens, None));
}

#[test]
fn casing_is_applied_so_a_capitalized_word_still_resolves() {
    let (_dir, mut content) = fixture();
    with_vocab(
        &mut content,
        &["[PAD]", "[UNK]", "[CLS]", "[SEP]", "\u{2581}rust"],
    );

    let tokenizer = from_gguf(&content).unwrap();
    let encoded = tokenizer.encode("Rust", false).unwrap();
    assert_eq!(encoded.get_tokens(), &["rust".to_owned()]);
}

#[test]
fn token_types_mark_which_entries_are_special() {
    // Without the type array the loader falls back to the bracket convention;
    // with it, a special token is identified by what the file says it is.
    let tokens = vec!["ctrl".to_owned(), "piece".to_owned()];
    // 3 is llama.cpp's CONTROL, 1 is NORMAL.
    let converted = vocab::to_wordpiece(&tokens, Some(&[3, 1]));
    assert_eq!(converted[0], "ctrl", "a control token is passed through");
    assert_eq!(converted[1], "##piece");
}
