//! One vector, as a WAL record.
//!
//! The log carries opaque payloads, so a field decides its own record shape.
//! This one is deliberately minimal: raw little-endian `f32`, nothing else.
//! Width is not stored because the field already knows its own dimension — a
//! record that disagreed would be from a different field, which the WAL's own
//! per-file separation makes unrepresentable.
//!
//! The framing, checksum and torn-tail detection all belong to the WAL
//! (`wal/frame.rs`); this is only what goes inside a frame.

/// Serialize a vector for the log.
pub(super) fn encode(vector: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(vector.len() * 4);
    for value in vector {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

/// Read a vector back, or `None` if the payload is not `dim` floats.
///
/// Returns `None` rather than erroring because replay is best-effort by
/// design: a record that cannot be a vector of this width is not one, and the
/// caller's job is to skip it rather than to abandon the whole recovery.
pub(super) fn decode(bytes: &[u8], dim: usize) -> Option<Vec<f32>> {
    if bytes.len() != dim * 4 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}

#[cfg(test)]
#[path = "record_test.rs"]
mod tests;
