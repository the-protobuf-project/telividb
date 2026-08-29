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
    /// `u32`, the usual encoding for the two numeric fields read here.
    pub(super) const U32: u32 = 4;
    /// `u64`. The specification permits any integer type for a metadata value,
    /// and a writer that chooses this one is not wrong — only less common.
    pub(super) const U64: u32 = 10;
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
                k if is_int(kind) && k.ends_with(".embedding_length") => {
                    dimensions = read_int(&mut cursor, kind);
                }
                k if is_int(kind) && k.ends_with(".context_length") => {
                    context_length = read_int(&mut cursor, kind);
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

/// Whether `kind` is an integer this reader accepts for a shape field.
fn is_int(kind: u32) -> bool {
    kind == tag::U32 || kind == tag::U64
}

/// Read a shape field, whichever integer width it was written as.
///
/// A `u64` too large for the field is dropped rather than truncated: a model
/// claiming four billion dimensions is a misread, and silently wrapping it to a
/// small number would be a shape the rest of the code would act on.
fn read_int(cursor: &mut Cursor<'_>, kind: u32) -> Option<u32> {
    match kind {
        tag::U64 => u32::try_from(cursor.u64()?).ok(),
        _ => cursor.u32(),
    }
}

/// Step over a metadata value of `kind`, returning false if the buffer ended.
fn skip(cursor: &mut Cursor<'_>, kind: u32) -> bool {
    if let Some(width) = width_of(kind) {
        return cursor.take(width).is_some();
    }
    match kind {
        tag::STRING => cursor.string().is_some(),
        tag::ARRAY => skip_array(cursor, 0),
        // An unknown tag means the rest of the header cannot be located, so
        // stop rather than resynchronise on what would be arbitrary bytes.
        _ => false,
    }
}

/// How deeply arrays may nest before the header is treated as malformed.
///
/// The bytes here come from a model host, so the nesting depth is not this
/// crate's to trust: each level costs twelve bytes to declare, and a two-megabyte
/// prefix could therefore declare well over a hundred thousand of them. Without
/// a bound the recursion below would overflow the stack on input that a server
/// is free to send. Eight is far past anything a real file uses.
const MAX_ARRAY_DEPTH: usize = 8;

/// Step over a counted array, including the vocabulary arrays.
fn skip_array(cursor: &mut Cursor<'_>, depth: usize) -> bool {
    if depth > MAX_ARRAY_DEPTH {
        return false;
    }
    let (Some(element), Some(count)) = (cursor.u32(), cursor.u64()) else {
        return false;
    };
    if element == tag::STRING {
        // Strings are individually length-prefixed, so this is the one case
        // that cannot be skipped with arithmetic.
        return (0..count).all(|_| cursor.string().is_some());
    }
    if element == tag::ARRAY {
        // Arrays nest, and the specification allows it. Each inner array
        // carries its own element tag and count, so the only way past one is to
        // read it. Rare in practice — but "rare" here meant the scan stopped at
        // the first one, losing every field after it, which for a file that
        // wrote its vocabulary early is the architecture itself.
        return (0..count).all(|_| skip_array(cursor, depth + 1));
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
