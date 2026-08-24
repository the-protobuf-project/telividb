//! WordPiece tokenization, built from the vocabulary inside the GGUF.
//!
//! Built from the GGUF rather than from a `tokenizer.json` beside it, because
//! rule 12 makes the file's digest the model's whole identity. A tokenizer
//! loaded from a separate file is unversioned by that digest — swap it and the
//! same "model" silently produces different token ids, and therefore different
//! vectors, with nothing anywhere detecting the change.

use crate::error::{Error, Result};
use candle_core::quantized::gguf_file::Content;
use tokenizers::models::wordpiece::WordPiece;
use tokenizers::normalizers::bert::BertNormalizer;
use tokenizers::pre_tokenizers::bert::BertPreTokenizer;
use tokenizers::processors::bert::BertProcessing;
use tokenizers::{AddedToken, Tokenizer};

/// Build a WordPiece tokenizer from `content`'s embedded vocabulary.
pub fn from_gguf(content: &Content) -> Result<Tokenizer> {
    let tokens = token_list(content)?;

    let model = WordPiece::builder()
        .vocab(vocab(&tokens)?)
        .unk_token(special(content, "tokenizer.ggml.unknown_token_id", &tokens))
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
        lowercase(content),
    )));
    tokenizer.with_pre_tokenizer(Some(BertPreTokenizer));

    let cls = special(content, "tokenizer.ggml.bos_token_id", &tokens);
    let sep = special(content, "tokenizer.ggml.eos_token_id", &tokens);
    if let (Some(cls_id), Some(sep_id)) = (id_of(&tokens, &cls), id_of(&tokens, &sep)) {
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

/// Whether the vocabulary is lowercase-only, and so whether to lowercase input.
///
/// Inferred from the vocabulary rather than read from a key, because GGUF has
/// no standard metadata for it. Feeding cased text to an uncased vocabulary
/// turns every capitalized word into `[UNK]` — common enough in real text to
/// wreck recall, and silent.
fn lowercase(content: &Content) -> bool {
    content
        .metadata
        .get("tokenizer.ggml.tokens")
        .and_then(|v| v.to_vec().ok())
        .map(|tokens| {
            !tokens
                .iter()
                .filter_map(|v| v.to_string().ok())
                .any(|t| t.chars().any(char::is_uppercase) && !t.starts_with('['))
        })
        .unwrap_or(true)
}

/// Position of `token` in the vocabulary, which is its id.
fn id_of(tokens: &[String], token: &str) -> Option<u32> {
    if token.is_empty() {
        return None;
    }
    tokens.iter().position(|t| t == token).map(|i| i as u32)
}

/// The vocabulary as an ordered list, where position *is* the token id.
fn token_list(content: &Content) -> Result<Vec<String>> {
    let values = content
        .metadata
        .get("tokenizer.ggml.tokens")
        .and_then(|v| v.to_vec().ok())
        .ok_or_else(|| Error::MissingFromGguf {
            what: "tokenizer.ggml.tokens".to_owned(),
        })?;

    values
        .iter()
        .map(|v| {
            v.to_string().map(|s| s.to_owned()).map_err(|_| {
                // A non-string entry would shift every id after it, so the
                // whole vocabulary is wrong rather than one token.
                Error::MissingFromGguf {
                    what: "a string-valued tokenizer.ggml.tokens entry".to_owned(),
                }
            })
        })
        .collect()
}

/// Turn the ordered list into the map `WordPiece` wants.
///
/// Routed through `read_bytes`, whose input format is one token per line with
/// the line number as the id — exactly what the list already is. That avoids
/// naming `ahash::AHashMap`, the map type the builder actually takes, and so
/// avoids taking a direct dependency on `ahash` purely to spell a type.
fn vocab(tokens: &[String]) -> Result<tokenizers::models::bpe::Vocab> {
    // The line-per-token encoding cannot represent a token containing a
    // newline, and `read_bytes` also trims trailing whitespace. Either would
    // silently shift or alter ids rather than fail, so both are refused here.
    // No BERT-family WordPiece vocabulary contains such a token.
    if let Some(bad) = tokens
        .iter()
        .find(|t| t.contains('\n') || t.trim_end() != t.as_str())
    {
        return Err(Error::Tokenizer(format!(
            "vocabulary entry {bad:?} contains a newline or trailing whitespace, \
             which this loader cannot represent without shifting token ids"
        )));
    }

    WordPiece::read_bytes(tokens.join("\n").as_bytes()).map_err(|e| Error::Tokenizer(e.to_string()))
}

/// Resolve a `*_token_id` metadata key to the token text it indexes.
///
/// Empty when the key is absent, which the caller reads as "this model
/// declares none" rather than as a failure — `unk` falls back to the BERT
/// default, and an absent BOS/EOS is simply not registered.
fn special(content: &Content, key: &str, tokens: &[String]) -> String {
    let id = content
        .metadata
        .get(key)
        .and_then(|v| v.to_u32().ok())
        .map(|v| v as usize);
    match id.and_then(|i| tokens.get(i)) {
        Some(token) => token.clone(),
        None if key.contains("unknown") => "[UNK]".to_owned(),
        None => String::new(),
    }
}

#[cfg(test)]
#[path = "tokenize_test.rs"]
mod tests;
