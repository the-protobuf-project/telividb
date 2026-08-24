//! One vector, as a WAL record.
//!
//! The log carries opaque payloads, so a field decides its own record shape:
//! a version byte, then raw little-endian `f32`. Width is not stored because
//! the field already knows its own dimension — a record disagreeing would be
//! from a different field, which the WAL's per-field separation makes
//! unrepresentable.
//!
//! **Why the version byte.** Rule 4 wants every on-disk structure versioned so
//! an unknown one is refused rather than guessed at. The frame around this
//! carries a length and a CRC (`wal/frame.rs`) but says nothing about what the
//! payload *means*, so without this a future encoding would be read as the
//! current one and silently produce wrong vectors.

/// Payload version. Bump when the encoding below changes.
const VERSION: u8 = 1;

/// Serialize a vector for the log.
pub(super) fn encode(vector: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + vector.len() * 4);
    out.push(VERSION);
    for value in vector {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

/// Read a vector back, or `None` if the payload is not a `VERSION` record of
/// `dim` floats.
///
/// Returns `None` rather than erroring because replay is best-effort by
/// design: a record that cannot be a vector of this width and version is not
/// one, and the caller's job is to account for it rather than to abandon the
/// whole recovery.
pub(super) fn decode(bytes: &[u8], dim: usize) -> Option<Vec<f32>> {
    let (&version, floats) = bytes.split_first()?;
    if version != VERSION || floats.len() != dim * 4 {
        return None;
    }
    Some(
        floats
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect(),
    )
}

#[cfg(test)]
#[path = "record_test.rs"]
mod tests;
