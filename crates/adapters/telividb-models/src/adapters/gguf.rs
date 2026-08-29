//! What a GGUF file says about itself.

use super::gguf_reader::Cursor;
use crate::{Error, Result};
use telividb_core::Architecture;

/// The metadata this crate needs from a GGUF header.
///
/// Read from a *prefix* of the file, so a model can be judged before it is
/// downloaded: a range request for the first couple of megabytes is enough to
/// learn the architecture and the vector width, which is everything needed to
/// decide whether fetching the rest is worthwhile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GgufHeader {
    /// The value of `general.architecture`, verbatim.
    ///
    /// Kept as written rather than parsed, so an unsupported file can be
    /// refused with a message naming what it actually is.
    pub architecture: String,
    /// Components per vector, if the header declared it.
    pub dimensions: Option<u32>,
    /// Maximum input length in tokens, if the header declared it.
    pub context_length: Option<u32>,
}

/// Metadata value tags, from the GGUF specification.
mod tag {
    /// `bool`, and the `u8`/`i8` scalars, are one byte each.
    pub(super) const ONE_BYTE: &[u32] = &[0, 1, 7];
    /// `u16` and `i16`.
    pub(super) const TWO_BYTE: &[u32] = &[2, 3];
    /// `u32`, `i32` and `f32`.
    pub(super) const FOUR_BYTE: &[u32] = &[4, 5, 6];
    /// `u32` specifically, which is how the two numeric fields here are typed.
    pub(super) const U32: u32 = 4;
    /// `u64`, `i64` and `f64`.
    pub(super) const EIGHT_BYTE: &[u32] = &[10, 11, 12];
    /// A length-prefixed string.
    pub(super) const STRING: u32 = 8;
    /// A typed, counted array.
    pub(super) const ARRAY: u32 = 9;
}

impl GgufHeader {
    /// How much of a file to read before its header can be judged.
    ///
    /// The fields this needs sit near the front, ahead of the tokenizer's
    /// vocabulary — which is the large part of the metadata and is skipped
    /// rather than parsed. Two megabytes covers every model in the catalog
    /// with room to spare; a file that needs more is reported as truncated
    /// rather than guessed at.
    pub const PREFIX_BYTES: u64 = 2 * 1024 * 1024;

    /// Read the header from the beginning of a file.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let pairs = cursor.header()?;

        let mut architecture = None;
        let mut dimensions = None;
        let mut context_length = None;

        for _ in 0..pairs {
            let Some(key) = cursor.string() else { break };
            let Some(kind) = cursor.u32() else { break };

            // Only three keys matter, and two of them are named after the
            // architecture — so the value is read when the key matches and
            // stepped over otherwise.
            match key.as_str() {
                "general.architecture" if kind == tag::STRING => {
                    architecture = cursor.string();
                }
                k if kind == tag::U32 && k.ends_with(".embedding_length") => {
                    dimensions = cursor.u32();
                }
                k if kind == tag::U32 && k.ends_with(".context_length") => {
                    context_length = cursor.u32();
                }
                _ => {
                    if !skip(&mut cursor, kind) {
                        break;
                    }
                }
            }

            if cursor.ended() {
                break;
            }
        }

        let Some(architecture) = architecture else {
            return Err(Error::Gguf(
                "the header declares no `general.architecture`, so there is no \
                 way to know which forward pass it needs"
                    .to_owned(),
            ));
        };
        Ok(Self {
            architecture: architecture.trim().to_owned(),
            dimensions,
            context_length,
        })
    }

    /// The architecture, if this engine implements it.
    pub fn supported(&self) -> Option<Architecture> {
        Architecture::from_gguf(&self.architecture)
    }

    /// Refuse a file this engine cannot read, naming what it is.
    ///
    /// `name` is what the caller was asking for — a repository, a URL, a
    /// catalog id — so the message points at something the person recognises
    /// rather than at a temporary path.
    pub fn require_supported(&self, name: &str) -> Result<Architecture> {
        self.supported()
            .ok_or_else(|| Error::UnsupportedArchitecture {
                name: name.to_owned(),
                found: self.architecture.clone(),
                supported: Architecture::NAMES.join(", "),
            })
    }
}

/// Step over a metadata value of `kind`, returning false if the buffer ended.
fn skip(cursor: &mut Cursor<'_>, kind: u32) -> bool {
    if let Some(width) = width_of(kind) {
        return cursor.take(width).is_some();
    }
    match kind {
        tag::STRING => cursor.string().is_some(),
        tag::ARRAY => skip_array(cursor),
        // An unknown tag means the rest of the header cannot be located, so
        // stop rather than resynchronise on what would be arbitrary bytes.
        _ => false,
    }
}

/// Step over a counted array, including the vocabulary arrays.
fn skip_array(cursor: &mut Cursor<'_>) -> bool {
    let (Some(element), Some(count)) = (cursor.u32(), cursor.u64()) else {
        return false;
    };
    if element == tag::STRING {
        // Strings are individually length-prefixed, so this is the one case
        // that cannot be skipped with arithmetic.
        return (0..count).all(|_| cursor.string().is_some());
    }
    let Some(width) = width_of(element) else {
        return false;
    };
    match usize::try_from(count)
        .ok()
        .and_then(|c| c.checked_mul(width))
    {
        Some(total) => cursor.take(total).is_some(),
        None => false,
    }
}

/// The fixed width of a scalar tag, or `None` for strings and arrays.
fn width_of(kind: u32) -> Option<usize> {
    [
        (tag::ONE_BYTE, 1),
        (tag::TWO_BYTE, 2),
        (tag::FOUR_BYTE, 4),
        (tag::EIGHT_BYTE, 8),
    ]
    .into_iter()
    .find_map(|(tags, width)| tags.contains(&kind).then_some(width))
}

#[cfg(test)]
#[path = "gguf_test.rs"]
mod tests;
