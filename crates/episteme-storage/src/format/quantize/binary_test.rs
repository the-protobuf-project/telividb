use super::*;

#[test]
fn signs_are_preserved() {
    let codes = BinaryCodes::encode(&[1.0, -1.0, 0.5, -0.5]);
    assert_eq!(codes.decode(), vec![1.0, -1.0, 1.0, -1.0]);
}

#[test]
fn magnitude_is_discarded() {
    // This is the trade: 32x compression buys direction only.
    let big = BinaryCodes::encode(&[100.0, -100.0]);
    let small = BinaryCodes::encode(&[0.001, -0.001]);
    assert_eq!(big, small);
}

#[test]
fn zero_encodes_as_positive_so_the_mapping_stays_total() {
    assert_eq!(BinaryCodes::encode(&[0.0]).decode(), vec![1.0]);
}

#[test]
fn compression_is_thirty_two_fold() {
    assert_eq!(BinaryCodes::encoded_len(768), 96);
    assert_eq!((768 * 4) / 96, 32);
}

#[test]
fn widths_that_are_not_multiples_of_eight_round_up() {
    assert_eq!(BinaryCodes::encoded_len(100), 13);
    let codes = BinaryCodes::encode(&vec![1.0; 100]);
    assert_eq!(codes.dim(), 100);
    assert_eq!(codes.decode().len(), 100);
}

#[test]
fn identical_vectors_have_zero_hamming_distance() {
    let a = BinaryCodes::encode(&[1.0, -1.0, 1.0, 1.0]);
    assert_eq!(hamming(&a, &a), Some(0));
}

#[test]
fn opposite_vectors_differ_in_every_bit() {
    let a = BinaryCodes::encode(&[1.0, 1.0, 1.0, 1.0]);
    let b = BinaryCodes::encode(&[-1.0, -1.0, -1.0, -1.0]);
    assert_eq!(hamming(&a, &b), Some(4));
}

#[test]
fn hamming_tracks_angular_similarity() {
    // The property that makes it usable as a first pass: a nearer vector must
    // have a smaller Hamming distance than a farther one.
    let query: Vec<f32> = (0..128).map(|i| (i as f32 * 0.1).sin()).collect();
    let near: Vec<f32> = query.iter().map(|x| x + 0.01).collect();
    let far: Vec<f32> = query.iter().map(|x| -x).collect();

    let q = BinaryCodes::encode(&query);
    assert!(
        hamming(&q, &BinaryCodes::encode(&near)) < hamming(&q, &BinaryCodes::encode(&far)),
        "binary codes did not preserve ordering"
    );
}

#[test]
fn mismatched_widths_are_refused_rather_than_compared() {
    let a = BinaryCodes::encode(&[1.0, 1.0]);
    let b = BinaryCodes::encode(&[1.0, 1.0, 1.0, 1.0]);
    assert_eq!(hamming(&a, &b), None);
}

#[test]
fn serialization_round_trips() {
    let v: Vec<f32> = (0..100)
        .map(|i| if i % 3 == 0 { 1.0 } else { -1.0 })
        .collect();
    let codes = BinaryCodes::encode(&v);
    let back = BinaryCodes::from_bytes(codes.as_bytes(), 100).unwrap();
    assert_eq!(back, codes);
}

#[test]
fn a_truncated_row_is_rejected() {
    let codes = BinaryCodes::encode(&vec![1.0; 64]);
    assert!(BinaryCodes::from_bytes(&codes.as_bytes()[..4], 64).is_none());
}
