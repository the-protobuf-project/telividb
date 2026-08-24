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
fn known_words_map_to_their_vocabulary_positions() {
    let (_dir, content) = fixture();
    let tokenizer = from_gguf(&content).unwrap();

    let ids = tokenizer.encode("cat", false).unwrap();
    // "cat" sits at index 9 in the fixture vocabulary.
    assert!(ids.get_ids().contains(&9), "got {:?}", ids.get_ids());
}

#[test]
fn an_unknown_word_becomes_the_declared_unk_token() {
    let (_dir, content) = fixture();
    let tokenizer = from_gguf(&content).unwrap();

    let ids = tokenizer.encode("zzzzqqq", false).unwrap();
    assert!(ids.get_ids().contains(&1), "got {:?}", ids.get_ids());
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
    let mut tokens: Vec<Value> = content
        .metadata
        .get("tokenizer.ggml.tokens")
        .unwrap()
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
fn an_absent_unk_id_falls_back_to_the_bert_default() {
    let (_dir, mut content) = fixture();
    content.metadata.remove("tokenizer.ggml.unknown_token_id");

    // Still builds: [UNK] is in the fixture vocabulary under its usual name.
    assert!(from_gguf(&content).is_ok());
}
