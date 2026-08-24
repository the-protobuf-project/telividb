//! Reading the vocabulary out of a GGUF, in the convention WordPiece wants.
//!
//! **The conversion this file exists for.** llama.cpp does not store a BERT
//! vocabulary the way BERT wrote it. WordPiece marks *continuations* with a
//! `##` prefix (`window` = `win` + `##dow`); the GGUF instead marks *word
//! starts* with `▁` (U+2581) and leaves continuations bare — SentencePiece's
//! convention, applied to a WordPiece vocabulary.
//!
//! Ignoring that does not fail loudly. Short words that happen to exist as
//! whole tokens still resolve, so tokenization *looks* like it works, while
//! every word needing a subword split silently becomes `[UNK]`. The resulting
//! embeddings are the right width, unit length, and nearly content-free.
//!
//! Token *ids* are untouched: an id is a position in the array, and only the
//! spelling of each entry changes.

use candle_core::quantized::gguf_file::Content;

/// Word-start marker used by the GGUF convention.
const WORD_START: char = '\u{2581}';

/// Continuation marker used by WordPiece.
const CONTINUATION: &str = "##";

/// llama.cpp's `NORMAL` token type. Anything else — unknown, control,
/// user-defined, unused, byte — is a special token whose spelling must be
/// left exactly as written.
const NORMAL: u32 = 1;

/// The vocabulary as written in the file, in id order.
pub fn raw_tokens(content: &Content) -> Option<Vec<String>> {
    let values = content
        .metadata
        .get("tokenizer.ggml.tokens")?
        .to_vec()
        .ok()?;
    values
        .iter()
        .map(|v| v.to_string().ok().cloned())
        .collect::<Option<Vec<String>>>()
}

/// Per-token type codes, when the file records them.
///
/// Written at either integer width depending on the converter's version, so
/// both are accepted rather than one guessed at.
pub fn token_types(content: &Content) -> Option<Vec<u32>> {
    let values = content
        .metadata
        .get("tokenizer.ggml.token_type")?
        .to_vec()
        .ok()?;
    Some(
        values
            .iter()
            .map(|v| {
                v.to_u32()
                    .or_else(|_| v.to_i32().map(|i| i as u32))
                    .unwrap_or(NORMAL)
            })
            .collect(),
    )
}

/// Rewrite `tokens` from the GGUF's convention into WordPiece's.
///
/// A word-start token loses its `▁`; a normal token without one is a
/// continuation and gains `##`. Special tokens are passed through untouched —
/// prefixing `[CLS]` would make it unmatchable.
pub fn to_wordpiece(tokens: &[String], types: Option<&[u32]>) -> Vec<String> {
    tokens
        .iter()
        .enumerate()
        .map(|(i, token)| {
            let is_normal = types.is_none_or(|t| t.get(i).copied().unwrap_or(NORMAL) == NORMAL);

            match token.strip_prefix(WORD_START) {
                Some(stripped) => stripped.to_owned(),
                // Without a type array, bracketed names are the only signal
                // that something is special — which is exactly the convention
                // every BERT vocabulary uses for them.
                None if !is_normal || token.starts_with('[') => token.clone(),
                None => format!("{CONTINUATION}{token}"),
            }
        })
        .collect()
}

/// Whether the vocabulary is lowercase-only, and so whether to lowercase input.
///
/// Inferred from the vocabulary because GGUF records no key for it, and
/// feeding cased text to an uncased vocabulary turns every capitalized word
/// into `[UNK]` — the first word of most sentences, and every proper noun.
///
/// Two exclusions, both of which produced exactly that failure:
///
/// **Special tokens are skipped.** `[CLS]`, `[SEP]`, `[UNK]`, `[PAD]` and
/// `[MASK]` are spelled in ASCII capitals in every BERT vocabulary ever
/// written, so counting them concludes that an entirely lowercase vocabulary
/// is cased.
///
/// **Only ASCII uppercase counts.** An uncased vocabulary still holds a
/// handful of non-ASCII uppercase codepoints — `ℝ` is in this very model —
/// and a general `is_uppercase` test finds them and reaches the same wrong
/// conclusion.
pub fn is_lowercase(tokens: &[String], types: Option<&[u32]>) -> bool {
    !tokens.iter().enumerate().any(|(i, token)| {
        let is_normal = types.is_none_or(|t| t.get(i).copied().unwrap_or(NORMAL) == NORMAL);
        is_normal && !token.starts_with('[') && token.chars().any(|c| c.is_ascii_uppercase())
    })
}

/// Whether `tokens` can be represented in the line-per-token vocabulary format.
///
/// A newline would shift every id after it and trailing whitespace would be
/// trimmed away, so both are refused rather than silently altering the
/// vocabulary.
pub fn unrepresentable(tokens: &[String]) -> Option<&String> {
    tokens
        .iter()
        .find(|t| t.contains('\n') || t.trim_end() != t.as_str())
}
