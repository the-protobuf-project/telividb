//! Converting a GGUF vocabulary into the convention WordPiece wants.
//!
//! Format knowledge, not runtime knowledge: these rules describe what
//! llama.cpp's converter writes, and they are the same whichever library read
//! the file. Carried over unchanged when the encoder moved to ggml, because
//! every one of them was established by a failure rather than by reading a
//! specification.
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

const WORD_START: char = '\u{2581}';

const CONTINUATION: &str = "##";

const NORMAL: u32 = 1;

/// Rewrite `tokens` from the GGUF's convention into WordPiece's.
///
/// A word-start token loses its `▁`; a normal token without one is a
/// *continuation* and gains `##`. Special tokens pass through untouched —
/// prefixing `[CLS]` would make it unmatchable.
///
/// Token ids are unaffected: an id is a position in the array, and only the
/// spelling of each entry changes.
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
/// Inferred from the vocabulary because GGUF records no key for it, and feeding
/// cased text to an uncased vocabulary turns every capitalised word into
/// `[UNK]` — the first word of most sentences, and every proper noun.
///
/// Two exclusions, both of which produced exactly that failure:
///
/// **Special tokens are skipped.** `[CLS]`, `[SEP]`, `[UNK]`, `[PAD]` and
/// `[MASK]` are spelled in ASCII capitals in every BERT vocabulary ever
/// written, so counting them concludes an entirely lowercase vocabulary is
/// cased.
///
/// **Only ASCII uppercase counts.** An uncased vocabulary still holds a handful
/// of non-ASCII uppercase codepoints — `ℝ` is in this very model — and a
/// general `is_uppercase` test finds them and reaches the same wrong
/// conclusion.
pub fn is_lowercase(tokens: &[String], types: Option<&[u32]>) -> bool {
    !tokens.iter().enumerate().any(|(i, token)| {
        let is_normal = types.is_none_or(|t| t.get(i).copied().unwrap_or(NORMAL) == NORMAL);
        is_normal && !token.starts_with('[') && token.chars().any(|c| c.is_ascii_uppercase())
    })
}

/// Whether `tokens` can be represented in the line-per-token vocabulary format.
///
/// A newline would shift every id after it, and trailing whitespace would be
/// trimmed away — both silently altering the vocabulary rather than failing, so
/// both are refused.
pub fn unrepresentable(tokens: &[String]) -> Option<&String> {
    tokens
        .iter()
        .find(|t| t.contains('\n') || t.trim_end() != t.as_str())
}
