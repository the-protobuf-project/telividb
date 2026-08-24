//! Turning wire types into Rust ones and back.

use crate::error::{Error, Result};
use telividb_proto::point::v1 as wire;

/// Bytes per `f32` on the wire.
const F32_BYTES: usize = 4;

/// Encode a vector for the wire.
///
/// Raw little-endian `f32` in a `bytes` field, never `repeated float`.
/// Protobuf encodes a repeated scalar element by element, so a 768-dimensional
/// vector would cost 768 varint operations per message — on the single
/// hottest path in the system. See CLAUDE.md's note on this.
pub fn to_wire(vector: &[f32]) -> wire::Vector {
    let mut data = Vec::with_capacity(vector.len() * F32_BYTES);
    for value in vector {
        data.extend_from_slice(&value.to_le_bytes());
    }
    wire::Vector {
        // `Bytes` rather than `Vec<u8>`: the generated types use it so a
        // decoded message can share the read buffer instead of copying every
        // vector out of it.
        data: data.into(),
        dimensions: vector.len() as i32,
    }
}

/// Decode a vector from the wire.
///
/// The declared width and the byte length must agree. A disagreement means
/// client and server disagree about the encoding itself, so it is reported
/// rather than truncated to whichever is shorter — a silently short vector
/// would score against reinterpreted bytes and rank plausibly wrongly.
pub fn from_wire(vector: &wire::Vector) -> Result<Vec<f32>> {
    let declared = vector.dimensions as usize;
    if vector.data.len() != declared * F32_BYTES {
        return Err(Error::Malformed {
            what: format!(
                "vector declares {declared} dimensions but carries {} bytes; expected {}",
                vector.data.len(),
                declared * F32_BYTES
            ),
        });
    }

    Ok(vector
        .data
        .as_chunks::<F32_BYTES>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect())
}

/// A point carrying one named vector, ready to send.
pub fn point_with_vector(field: &str, vector: &[f32]) -> wire::Point {
    wire::Point {
        name: String::new(),
        vectors: vec![wire::NamedVector {
            field_id: field.to_owned(),
            vector: Some(to_wire(vector)),
            text: String::new(),
        }],
        span: None,
        content_ref: None,
    }
}

/// A point whose vector the server will compute from `text`.
///
/// The counterpart to [`point_with_vector`]: exactly one of the two fields is
/// set, and the server refuses both or neither.
pub fn point_from_text(field: &str, text: &str) -> wire::Point {
    wire::Point {
        name: String::new(),
        vectors: vec![wire::NamedVector {
            field_id: field.to_owned(),
            vector: None,
            text: text.to_owned(),
        }],
        span: None,
        // The same text is stored as content as well as sent for embedding.
        // Without it a search result is a bare id, and the caller has to
        // resolve it against their own storage before it means anything.
        content_ref: Some(wire::ContentRef {
            uri: String::new(),
            range_start: 0,
            range_end: 0,
            sha256: Default::default(),
            inline_text: text.to_owned(),
        }),
    }
}

/// The text a point carries inline, if any.
///
/// `inline_text` rather than fetching the URI: the database stores content
/// *references*, not media (invariant 19), so resolving one is the caller's
/// job and their storage's — not something an SDK should do behind their back.
pub fn inline_text(point: &wire::Point) -> Option<String> {
    point
        .content_ref
        .as_ref()
        .map(|r| r.inline_text.clone())
        .filter(|t| !t.is_empty())
}

#[cfg(test)]
#[path = "convert_test.rs"]
mod tests;
