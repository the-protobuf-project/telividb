//! Building the tokenizer from the vocabulary inside the GGUF.
//!
//! From the model file rather than a `tokenizer.json` beside it, because rule
//! 12 makes the file's digest the model's whole identity. A tokenizer loaded
//! from a separate file is unversioned by that digest — swap it and the same
//! "model" silently produces different token ids, and therefore different
//! vectors, with nothing detecting the change.
//!
//! The conversion rules live in [`super::vocab_rules`]; this is the assembly.

use crate::error::{Error, Result};
use std::path::Path;
use telividb_compute::Header;
use tokenizers::models::wordpiece::WordPiece;
use tokenizers::normalizers::bert::BertNormalizer;
use tokenizers::pre_tokenizers::bert::BertPreTokenizer;
use tokenizers::processors::bert::BertProcessing;
use tokenizers::{AddedToken, Tokenizer};

/// Read `path`'s vocabulary and build a WordPiece tokenizer from it.
///
/// Header only — no tensor data, no device allocation. A vocabulary is a string
/// array, and paying a model's full residency to read one would be absurd.
pub fn from_gguf(path: &Path) -> Result<Tokenizer> {
    let header = Header::open(path).map_err(|e| Error::Compute(e.to_string()))?;
    from_header(&header)
}

/// Build the tokenizer from an already-parsed header.
pub fn from_header(weights: &Header) -> Result<Tokenizer> {
    let raw = weights
        .str_array("tokenizer.ggml.tokens")
        .ok_or_else(|| Error::MissingFromGguf {
            what: "tokenizer.ggml.tokens".to_owned(),
        })?;
    let types: Option<Vec<u32>> = weights
        .i32_array("tokenizer.ggml.token_type")
        .map(|v| v.into_iter().map(|t| t as u32).collect());
    let tokens = super::vocab_rules::to_wordpiece(&raw, types.as_deref());

    let model = WordPiece::builder()
        .vocab(build_vocab(&tokens)?)
        .unk_token(special(&raw, weights, "tokenizer.ggml.unknown_token_id"))
        .continuing_subword_prefix("##".to_owned())
        .build()
        .map_err(|e| Error::Tokenizer(e.to_string()))?;

    let mut tokenizer = Tokenizer::new(model);

    // **The pipeline, not just the vocabulary.** A bare `Tokenizer::new` has no
    // normalizer and no pre-tokenizer, so the whole input arrives at WordPiece
    // as a single "word" — which matches nothing and collapses to one `[UNK]`.
    // Every text then embeds identically, and nothing errors: the vectors are
    // the right width, finite, and completely uninformative.
    tokenizer.with_normalizer(Some(BertNormalizer::new(
        true,
        true,
        // `None` defers to `lowercase`, which is what an uncased model wants
        // and is harmless for a cased one that never sees an accent stripped.
        None,
        super::vocab_rules::is_lowercase(&raw, types.as_deref()),
    )));
    tokenizer.with_pre_tokenizer(Some(BertPreTokenizer));

    let cls = special(&raw, weights, "tokenizer.ggml.bos_token_id");
    let sep = special(&raw, weights, "tokenizer.ggml.eos_token_id");
    if let (Some(cls_id), Some(sep_id)) = (id_of(&raw, &cls), id_of(&raw, &sep)) {
        // Registered as special so they survive normalization and are never
        // split into subwords — a `[CLS]` tokenized as `[`, `cls`, `]` would
        // put three wrong ids where the model expects one.
        tokenizer.add_special_tokens(&[
            AddedToken::from(cls.clone(), true),
            AddedToken::from(sep.clone(), true),
        ]);
        tokenizer.with_post_processor(Some(BertProcessing::new((sep, sep_id), (cls, cls_id))));
    }
    Ok(tokenizer)
}

/// Turn the ordered list into the map `WordPiece` wants.
fn build_vocab(tokens: &[String]) -> Result<tokenizers::models::bpe::Vocab> {
    if let Some(bad) = super::vocab_rules::unrepresentable(tokens) {
        return Err(Error::Tokenizer(format!(
            "vocabulary entry {bad:?} contains a newline or trailing whitespace, \
             which this loader cannot represent without shifting token ids"
        )));
    }
    WordPiece::read_bytes(tokens.join("\n").as_bytes()).map_err(|e| Error::Tokenizer(e.to_string()))
}

/// Resolve a `*_token_id` metadata key to the token text it indexes.
fn special(raw: &[String], weights: &Header, key: &str) -> String {
    let id = weights.u32_meta(key).map(|v| v as usize);
    match id.and_then(|i| raw.get(i)) {
        Some(token) => token.clone(),
        None if key.contains("unknown") => "[UNK]".to_owned(),
        None => String::new(),
    }
}

/// Position of `token` in the vocabulary, which is its id.
fn id_of(tokens: &[String], token: &str) -> Option<u32> {
    if token.is_empty() {
        return None;
    }
    tokens.iter().position(|t| t == token).map(|i| i as u32)
}
