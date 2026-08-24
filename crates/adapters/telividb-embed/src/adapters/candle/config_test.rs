use super::*;
use crate::adapters::candle::fixture::{TinyModel, write_tiny_gguf};
use candle_core::quantized::gguf_file::Content;

fn read(path: &std::path::Path) -> Content {
    let mut file = std::fs::File::open(path).unwrap();
    Content::read(&mut file).unwrap()
}

#[test]
fn a_fixture_gguf_reads_back_the_shape_it_was_written_with() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    let model = TinyModel::default();
    write_tiny_gguf(&path, &model).unwrap();

    let config = BertConfig::from_gguf(&read(&path)).unwrap();
    assert_eq!(config.arch, "bert");
    assert_eq!(config.layers, model.layers);
    assert_eq!(config.hidden, model.hidden);
    assert_eq!(config.ff, model.ff);
    assert_eq!(config.heads, model.heads);
    assert_eq!(config.context, model.context);
    assert_eq!(config.head_dim(), model.hidden / model.heads);
}

#[test]
fn an_unsupported_architecture_is_refused_rather_than_run() {
    // A mismatched architecture finds tensors by the names it expects, runs to
    // completion, and returns wrong vectors. Refusing at load is the only
    // point where it is still detectable.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("llama.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let mut content = read(&path);
    content.metadata.insert(
        "general.architecture".to_owned(),
        candle_core::quantized::gguf_file::Value::String("llama".to_owned()),
    );

    match BertConfig::from_gguf(&content) {
        Err(crate::Error::UnsupportedArchitecture { found, .. }) => assert_eq!(found, "llama"),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

#[test]
fn a_missing_hyperparameter_names_the_key_it_needed() {
    // Guessing a default here produces a model that runs and is wrong, so the
    // error has to be specific enough to fix.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let mut content = read(&path);
    content.metadata.remove("bert.block_count");

    match BertConfig::from_gguf(&content) {
        Err(crate::Error::MissingFromGguf { what }) => assert_eq!(what, "bert.block_count"),
        other => panic!("expected a missing-key error, got {other:?}"),
    }
}

#[test]
fn a_head_count_that_does_not_divide_the_hidden_width_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let mut content = read(&path);
    content.metadata.insert(
        "bert.attention.head_count".to_owned(),
        candle_core::quantized::gguf_file::Value::U32(3),
    );

    assert!(matches!(
        BertConfig::from_gguf(&content),
        Err(crate::Error::MissingFromGguf { .. })
    ));
}

#[test]
fn an_integer_written_at_any_width_is_accepted() {
    // GGUF writers are not consistent about integer width for the same key,
    // and matching on one type would reject files that are entirely valid.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let mut content = read(&path);
    content.metadata.insert(
        "bert.block_count".to_owned(),
        candle_core::quantized::gguf_file::Value::U64(2),
    );

    assert_eq!(BertConfig::from_gguf(&content).unwrap().layers, 2);
}

#[test]
fn a_declared_pooling_mode_is_read_from_the_file() {
    // The model knows; asking the caller invites a wrong guess that produces
    // right-shaped, badly-ranked vectors with no error anywhere.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let mut content = read(&path);
    content.metadata.insert(
        "bert.pooling_type".to_owned(),
        candle_core::quantized::gguf_file::Value::U32(2),
    );

    let config = BertConfig::from_gguf(&content).unwrap();
    assert_eq!(config.pooling, Some(crate::Pooling::Cls));
}

#[test]
fn an_unpooled_mode_is_reported_as_no_declaration_rather_than_guessed() {
    // 0 is "no pooling" and 3/4 are last-token and rerank — none of them is a
    // pooled sentence embedding, so the honest answer is that the file did
    // not declare one and the caller must decide.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let mut content = read(&path);
    for mode in [0u32, 3, 4] {
        content.metadata.insert(
            "bert.pooling_type".to_owned(),
            candle_core::quantized::gguf_file::Value::U32(mode),
        );
        assert_eq!(BertConfig::from_gguf(&content).unwrap().pooling, None);
    }
}

#[test]
fn rope_is_detected_from_the_files_own_metadata() {
    // A rotary model carries no position table at all, so this is what
    // decides whether the loader looks for one.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let plain = BertConfig::from_gguf(&read(&path)).unwrap();
    assert!(!plain.uses_rope(), "classic BERT has learned positions");

    let mut content = read(&path);
    content.metadata.insert(
        "bert.rope.freq_base".to_owned(),
        candle_core::quantized::gguf_file::Value::F32(1000.0),
    );
    assert!(BertConfig::from_gguf(&content).unwrap().uses_rope());
}
