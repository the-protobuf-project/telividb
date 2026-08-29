//! Between a wire vector and `f32`s.
//!
//! Here rather than beside the point conversions because these operate on a
//! *vector*, and both sides of the system need them: the point service encodes
//! stored vectors into responses, and the embedding path encodes ones it has
//! just computed. Sharing one definition is what keeps the two byte-compatible.
//!
//! Vectors travel as `bytes` rather than `repeated float`: protobuf encodes
//! each element separately, which is 768 varint operations per message on the
//! hot path.

use telividb_buffers::protobuf::point::v1::Vector as WireVector;
use tonic::Status;

/// Decode a wire vector's raw little-endian `f32` payload.
///
/// Refuses a negative or mismatched dimension count rather than trusting the
/// header: the length is the caller's claim about bytes the caller also sent,
/// and a wrong one would reinterpret the payload rather than fail.
pub(crate) fn vector_to_domain(wire: &WireVector) -> Result<Vec<f32>, Status> {
    if wire.dimensions < 0 {
        return Err(Status::invalid_argument(
            "vector dimensions must not be negative",
        ));
    }
    let declared = wire.dimensions as usize;
    // Checked: a large declared width would otherwise wrap and accidentally
    // match a short payload.
    let expected = declared
        .checked_mul(4)
        .ok_or_else(|| Status::invalid_argument("vector dimensions overflow"))?;
    if wire.data.len() != expected {
        return Err(Status::invalid_argument(format!(
            "vector declares {declared} dimensions but carries {} bytes; expected {expected}",
            wire.data.len(),
        )));
    }
    Ok(wire
        .data
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect())
}

/// The reverse of [`vector_to_domain`].
pub(crate) fn vector_to_wire(vector: &[f32]) -> WireVector {
    let mut data = Vec::with_capacity(vector.len() * 4);
    for value in vector {
        data.extend_from_slice(&value.to_le_bytes());
    }
    WireVector {
        data: data.into(),
        dimensions: vector.len() as i32,
    }
}
