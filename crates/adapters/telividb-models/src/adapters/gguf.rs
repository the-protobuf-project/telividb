//! What a GGUF file says about itself.

use super::gguf_reader::Cursor;
use super::gguf_skip::{is_int, read_int, skip};
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
pub(super) mod tag {
    /// `bool`, and the `u8`/`i8` scalars, are one byte each.
    pub(crate) const ONE_BYTE: &[u32] = &[0, 1, 7];
    /// `u16` and `i16`.
    pub(crate) const TWO_BYTE: &[u32] = &[2, 3];
    /// `u32`, `i32` and `f32`.
    pub(crate) const FOUR_BYTE: &[u32] = &[4, 5, 6];
    /// `u32`, the usual encoding for the two numeric fields read here.
    pub(crate) const U32: u32 = 4;
    /// `u64`. The specification permits any integer type for a metadata value,
    /// and a writer that chooses this one is not wrong — only less common.
    pub(crate) const U64: u32 = 10;
    /// `u64`, `i64` and `f64`.
    pub(crate) const EIGHT_BYTE: &[u32] = &[10, 11, 12];
    /// A length-prefixed string.
    pub(crate) const STRING: u32 = 8;
    /// A typed, counted array.
    pub(crate) const ARRAY: u32 = 9;
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

#[cfg(test)]
#[path = "gguf_test.rs"]
mod tests;

#[cfg(test)]
#[path = "gguf_encoding_test.rs"]
mod encoding_tests;
