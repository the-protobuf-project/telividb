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
fn an_empty_payload_decodes_only_at_dim_zero() {
    assert_eq!(decode(&[], 0), Some(Vec::new()));
    assert_eq!(decode(&[], 3), None);
}
