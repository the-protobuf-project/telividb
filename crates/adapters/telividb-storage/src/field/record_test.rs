use super::*;

#[test]
fn a_vector_round_trips() {
    let v = vec![1.0f32, -2.5, 3.25, 0.0];
    assert_eq!(decode(&encode(&v), 4), Some(v));
}

#[test]
fn a_payload_of_the_wrong_width_is_refused() {
    // Not an error: replay skips what cannot be a vector of this width rather
    // than abandoning recovery of everything after it.
    let v = vec![1.0f32, 2.0];
    assert_eq!(decode(&encode(&v), 4), None);
    assert_eq!(decode(&encode(&v), 1), None);
}

#[test]
fn a_versioned_empty_vector_round_trips() {
    assert_eq!(decode(&encode(&[]), 0), Some(Vec::new()));
    assert_eq!(decode(&encode(&[]), 3), None);
}

#[test]
fn an_unknown_version_is_refused() {
    // Without the version byte a future encoding would be read as this one and
    // silently produce wrong vectors.
    let mut bytes = encode(&[1.0f32, 2.0]);
    bytes[0] = 99;
    assert_eq!(decode(&bytes, 2), None);
}

#[test]
fn an_empty_payload_has_no_version_to_read() {
    assert_eq!(decode(&[], 0), None);
}
