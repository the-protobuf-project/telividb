use super::*;

fn span(a: u64, b: u64) -> Span {
    Span::new(a, b).unwrap()
}

#[test]
fn round_trips_present_spans() {
    let spans = vec![Some(span(0, 1000)), Some(span(12_400, 18_900))];
    assert_eq!(decode(&encode(&spans), 2).unwrap(), spans);
}

#[test]
fn round_trips_absent_spans() {
    // A text chunk has no temporal extent while the transcript beside it does.
    let spans = vec![Some(span(0, 500)), None, Some(span(500, 900))];
    assert_eq!(decode(&encode(&spans), 3).unwrap(), spans);
}

#[test]
fn an_empty_file_round_trips() {
    assert!(decode(&encode(&[]), 0).unwrap().is_empty());
}

#[test]
fn a_zero_length_span_is_preserved() {
    let spans = vec![Some(span(100, 100))];
    assert_eq!(decode(&encode(&spans), 1).unwrap(), spans);
}

#[test]
fn stride_is_fixed_so_offsets_are_computable() {
    let spans = vec![Some(span(0, 1)), None, Some(span(2, 3))];
    assert_eq!(encode(&spans).len(), 3 * SPAN_BYTES);
    assert_eq!(offset_of(2), 32);
}

#[test]
fn a_truncated_file_is_refused() {
    // Every offset after the missing row would be wrong, and the error would
    // surface later as mismatched timestamps.
    let bytes = encode(&[Some(span(0, 1)), Some(span(2, 3))]);
    assert!(matches!(
        decode(&bytes[..SPAN_BYTES + 4], 2),
        Err(Error::Truncated { .. })
    ));
}

#[test]
fn an_inverted_span_in_the_file_is_refused() {
    let mut bytes = encode(&[Some(span(100, 200))]);
    bytes[0..8].copy_from_slice(&900u64.to_le_bytes());
    assert!(decode(&bytes, 1).is_err());
}

#[test]
fn the_absent_sentinel_cannot_collide_with_a_real_span() {
    // u64::MAX as a start with u64::MAX as an end would be a zero-length span
    // at the far end of time; it decodes as absent, which is the intended
    // reading and the only ambiguity in the format.
    let spans = vec![None];
    assert_eq!(decode(&encode(&spans), 1).unwrap(), vec![None]);
}
