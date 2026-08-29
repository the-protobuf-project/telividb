//! Headers written in ways the common case is not.
//!
//! Separate from `gguf_test.rs` because that file asks whether a well-formed
//! header is read correctly, and this one asks whether a *legal but unusual*
//! one is read at all. The specification is wider than what any single writer
//! emits, and both of these were found by reading it rather than by a failure:
//! each one silently lost a field rather than erroring.
//!
//! These build their bytes by hand rather than through the fixture next door,
//! because the encoding is the thing under test.

use super::GgufHeader;

/// A metadata value written as `u64` rather than the usual `u32`.
///
/// The GGUF specification allows any integer type for a metadata value, so a
/// writer choosing this one is unusual rather than wrong.
fn u64_value(out: &mut Vec<u8>, value: u64) {
    out.extend(10u32.to_le_bytes());
    out.extend(value.to_le_bytes());
}

#[test]
fn a_shape_field_written_as_u64_is_still_read() {
    // Before this was handled the arm did not match, the value was skipped, and
    // `dimensions` came back `None` — so a perfectly good model reported no
    // width and the catalog check that compares against it failed.
    let mut bytes = b"GGUF".to_vec();
    bytes.extend(3u32.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());
    bytes.extend(2u64.to_le_bytes());

    for (key, value) in [
        ("general.architecture", None),
        ("bert.embedding_length", Some(768u64)),
    ] {
        bytes.extend((key.len() as u64).to_le_bytes());
        bytes.extend(key.as_bytes());
        match value {
            Some(v) => u64_value(&mut bytes, v),
            None => {
                bytes.extend(8u32.to_le_bytes());
                bytes.extend(4u64.to_le_bytes());
                bytes.extend(b"bert");
            }
        }
    }

    let header = GgufHeader::parse(&bytes).expect("a u64-encoded header");
    assert_eq!(header.architecture, "bert");
    assert_eq!(header.dimensions, Some(768));
}

#[test]
fn a_nested_array_is_stepped_over_rather_than_stopping_the_scan() {
    // The specification allows arrays of arrays. Before this was handled the
    // scan stopped at the first one — so every field after it was lost, which
    // for a file that writes its vocabulary early is the architecture itself.
    let mut bytes = b"GGUF".to_vec();
    bytes.extend(3u32.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());
    bytes.extend(3u64.to_le_bytes());

    // A nested array first, so anything read after it proves the skip worked.
    let key = "some.nested";
    bytes.extend((key.len() as u64).to_le_bytes());
    bytes.extend(key.as_bytes());
    bytes.extend(9u32.to_le_bytes()); // array
    bytes.extend(9u32.to_le_bytes()); // of arrays
    bytes.extend(2u64.to_le_bytes()); // two of them
    for _ in 0..2 {
        bytes.extend(4u32.to_le_bytes()); // of u32
        bytes.extend(3u64.to_le_bytes()); // three each
        for v in 0..3u32 {
            bytes.extend(v.to_le_bytes());
        }
    }

    let (key, value) = ("general.architecture", "bert");
    bytes.extend((key.len() as u64).to_le_bytes());
    bytes.extend(key.as_bytes());
    bytes.extend(8u32.to_le_bytes());
    bytes.extend((value.len() as u64).to_le_bytes());
    bytes.extend(value.as_bytes());
    let key = "bert.embedding_length";
    bytes.extend((key.len() as u64).to_le_bytes());
    bytes.extend(key.as_bytes());
    bytes.extend(4u32.to_le_bytes());
    bytes.extend(384u32.to_le_bytes());

    let header = GgufHeader::parse(&bytes).expect("a header with a nested array");
    assert_eq!(header.architecture, "bert");
    assert_eq!(
        header.dimensions,
        Some(384),
        "the scan stopped at the nested array"
    );
}
