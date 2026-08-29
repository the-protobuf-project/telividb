//! Byte-level BPE, for the Qwen3 and Llama families.
//!
//! A second tokenizer rather than a variation on the first, because the two
//! share nothing: WordPiece splits on punctuation and whitespace and matches
//! `##`-prefixed continuations against a vocabulary; byte-level BPE maps the
//! raw bytes into a printable alphabet and merges pairs in a learned order.
//!
//! Getting this wrong is the quiet failure described in [`super::vocab`] —
//! every text collapsing to the same few ids, producing vectors that are the
//! right width, finite, and uninformative. There is no error to notice, which
//! is why the tokenizer is built from the model's own file (rule 12) rather
//! than assumed.

use crate::error::{Error, Result};
use telividb_compute::Header;
use tokenizers::Tokenizer;
use tokenizers::decoders::byte_level::ByteLevel as ByteLevelDecoder;
use tokenizers::models::bpe::{BPE, Vocab};
use tokenizers::pre_tokenizers::byte_level::ByteLevel;
use tokenizers::processors::template::TemplateProcessing;

/// Build a byte-level BPE tokenizer from a GGUF header.
pub(super) fn build(weights: &Header, tokens: &[String]) -> Result<Tokenizer> {
    let merges =
        weights
            .str_array("tokenizer.ggml.merges")
            .ok_or_else(|| Error::MissingFromGguf {
                what: "tokenizer.ggml.merges (a BPE vocabulary without its merge \
                   order cannot reproduce the model's tokenization)"
                    .to_owned(),
            })?;

    let vocab: Vocab = tokens
        .iter()
        .enumerate()
        .map(|(id, token)| (token.clone(), id as u32))
        .collect();

    // Each merge is a space-separated pair. A malformed line is skipped rather
    // than failing the load: the merge order is a ranked list, and one missing
    // rank degrades a rare word, where refusing the model helps nobody.
    let merges: Vec<(String, String)> = merges
        .iter()
        .filter_map(|line| line.split_once(' '))
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .collect();
    if merges.is_empty() {
        return Err(Error::Tokenizer(
            "the merge list is present but empty, so every word would tokenize \
             one byte at a time"
                .to_owned(),
        ));
    }

    let model = BPE::builder()
        .vocab_and_merges(vocab, merges)
        .build()
        .map_err(|e| Error::Tokenizer(e.to_string()))?;

    let mut tokenizer = Tokenizer::new(model);

    // `add_prefix_space: false`. The GGUF vocabulary already encodes a leading
    // space as `Ġ`, so prepending one here would shift the first word onto a
    // different token than the model was trained on.
    tokenizer.with_pre_tokenizer(Some(ByteLevel::new(false, true, true)));
    tokenizer.with_decoder(Some(ByteLevelDecoder::default()));

    if let Some(processor) = eos_processor(weights, tokens) {
        tokenizer.with_post_processor(Some(processor));
    }
    Ok(tokenizer)
}

/// Append the end-of-text token, when the model asks for one.
///
/// Not cosmetic for this family. These models pool the **last** token
/// ([`Pooling::Last`](crate::domain::Pooling::Last)), and they were trained
/// with the sequence terminated — so the position being read is the one after
/// the text. Omitting it reads the final word's state instead of the summary
/// state, which is a worse vector rather than a broken one.
///
/// Returns `None` when the header does not ask for it, rather than appending
/// anyway: `add_eos_token` is the model speaking.
fn eos_processor(weights: &Header, tokens: &[String]) -> Option<TemplateProcessing> {
    if weights.bool_meta("tokenizer.ggml.add_eos_token") != Some(true) {
        return None;
    }
    let id = weights.u32_meta("tokenizer.ggml.eos_token_id")?;
    let text = tokens.get(id as usize)?.clone();

    TemplateProcessing::builder()
        .try_single(format!("$A:0 {text}:0"))
        .ok()?
        .special_tokens(vec![(text, id)])
        .build()
        .ok()
}
