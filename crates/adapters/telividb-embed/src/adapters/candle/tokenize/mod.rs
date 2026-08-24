//! WordPiece tokenization, built from the vocabulary inside the GGUF.
//!
//! Built from the GGUF rather than from a `tokenizer.json` beside it, because
//! rule 12 makes the file's digest the model's whole identity. A tokenizer
//! loaded from a separate file is unversioned by that digest — swap it and the
//! same "model" silently produces different token ids, and therefore different
//! vectors, with nothing anywhere detecting the change.
//!
//! See [`vocab`] for the convention conversion this depends on.

mod vocab;

use crate::error::{Error, Result};
use candle_core::quantized::gguf_file::Content;
use tokenizers::models::wordpiece::WordPiece;
use tokenizers::normalizers::bert::BertNormalizer;
use tokenizers::pre_tokenizers::bert::BertPreTokenizer;
use tokenizers::processors::bert::BertProcessing;
use tokenizers::{AddedToken, Tokenizer};

/// Build a WordPiece tokenizer from `content`'s embedded vocabulary.
pub fn from_gguf(content: &Content) -> Result<Tokenizer> {
    let raw = vocab::raw_tokens(content).ok_or_else(|| Error::MissingFromGguf {
        what: "tokenizer.ggml.tokens".to_owned(),
    })?;
    let types = vocab::token_types(content);
    let tokens = vocab::to_wordpiece(&raw, types.as_deref());

    let model = WordPiece::builder()
        .vocab(build_vocab(&tokens)?)
        .unk_token(special(&raw, content, "tokenizer.ggml.unknown_token_id"))
        .continuing_subword_prefix("##".to_owned())
        .build()
        .map_err(|e| Error::Tokenizer(e.to_string()))?;

    let mut tokenizer = Tokenizer::new(model);

    // **The pipeline, not just the vocabulary.** A bare `Tokenizer::new` has no
    // normalizer and no pre-tokenizer, so the entire input arrives at WordPiece
    // as a single "word" — which matches nothing and collapses to one `[UNK]`.
    // Every text then embeds identically, and nothing errors: the vectors are
    // the right width, finite, and completely uninformative.
    tokenizer.with_normalizer(Some(BertNormalizer::new(
        true,
        true,
        // `None` defers to `lowercase`, which is what an uncased model wants
        // and is harmless for a cased one that never sees an accent stripped.
        None,
        vocab::is_lowercase(&raw, types.as_deref()),
    )));
    tokenizer.with_pre_tokenizer(Some(BertPreTokenizer));

    let cls = special(&raw, content, "tokenizer.ggml.bos_token_id");
    let sep = special(&raw, content, "tokenizer.ggml.eos_token_id");
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
///
/// Routed through `read_bytes`, whose input format is one token per line with
/// the line number as the id — exactly what the list already is. That avoids
/// naming `ahash::AHashMap`, the map type the builder takes, and so avoids a
/// direct dependency on `ahash` purely to spell a type.
fn build_vocab(tokens: &[String]) -> Result<tokenizers::models::bpe::Vocab> {
    if let Some(bad) = vocab::unrepresentable(tokens) {
        return Err(Error::Tokenizer(format!(
            "vocabulary entry {bad:?} contains a newline or trailing whitespace, \
             which this loader cannot represent without shifting token ids"
        )));
    }

    WordPiece::read_bytes(tokens.join("\n").as_bytes()).map_err(|e| Error::Tokenizer(e.to_string()))
}

/// Resolve a `*_token_id` metadata key to the token text it indexes.
///
/// Read from the **raw** vocabulary, not the rewritten one: a special token is
/// passed through the conversion untouched, so both agree, and reading the
/// original keeps that independent of how the rewrite treats them.
fn special(raw: &[String], content: &Content, key: &str) -> String {
    let id = content
        .metadata
        .get(key)
        .and_then(|v| v.to_u32().ok())
        .map(|v| v as usize);
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

#[cfg(test)]
#[path = "tokenize_test.rs"]
mod tests;
