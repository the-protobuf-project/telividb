use super::*;

#[test]
fn compression_is_exactly_two_fold() {
    assert_eq!(F16Row::encoded_len(768), 1536);
    assert_eq!((768 * 4) / F16Row::encoded_len(768), 2);
}

#[test]
fn common_values_round_trip_exactly() {
    // Powers of two and small integers are representable, so they must survive
    // untouched rather than merely closely.
    for v in [0.0f32, 1.0, -1.0, 0.5, 2.0, -0.25, 1024.0] {
        let back = F16Row::encode(&[v]).decode()[0];
        assert_eq!(back, v, "{v} did not round trip exactly");
    }
}

#[test]
fn relative_error_stays_near_three_decimal_digits() {
    let v: Vec<f32> = (1..=512).map(|i| i as f32 * 0.013).collect();
    let back = F16Row::encode(&v).decode();
    let worst = v
        .iter()
        .zip(&back)
        .map(|(a, b)| ((a - b) / a).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-3,
        "relative error {worst} is worse than expected"
    );
}

#[test]
fn signs_are_preserved_including_negative_zero() {
    assert!(F16Row::encode(&[-0.0]).decode()[0].is_sign_negative());
    assert!(F16Row::encode(&[-7.5]).decode()[0] < 0.0);
}

#[test]
fn overflow_saturates_to_infinity_rather_than_wrapping() {
    // Loud, not silent: an infinite score propagates visibly, where a wrapped
    // value would quietly rank the vector somewhere near zero.
    let back = F16Row::encode(&[1e30]).decode()[0];
    assert!(back.is_infinite() && back > 0.0, "got {back}");
    assert!(F16Row::encode(&[-1e30]).decode()[0].is_infinite());
}

#[test]
fn the_largest_finite_value_still_round_trips() {
    let max = F16Row::max_finite();
    assert!(max > 65_000.0 && max.is_finite());
    assert_eq!(F16Row::encode(&[max]).decode()[0], max);
}

#[test]
fn very_small_values_flush_to_zero() {
    // Harmless for a normalized embedding component; worth knowing for one
    // that is not.
    assert!(F16Row::min_positive() > 0.0);
    assert_eq!(F16Row::encode(&[1e-12]).decode()[0], 0.0);
}

#[test]
fn subnormals_survive() {
    // The case a hand-rolled conversion usually gets wrong.
    let tiny = F16Row::min_positive();
    assert_eq!(F16Row::encode(&[tiny]).decode()[0], tiny);
}

#[test]
fn nan_stays_nan() {
    assert!(F16Row::encode(&[f32::NAN]).decode()[0].is_nan());
}

#[test]
fn serialization_round_trips() {
    let v: Vec<f32> = (0..64).map(|i| i as f32 * 0.3 - 5.0).collect();
    let row = F16Row::encode(&v);
    let mut bytes = Vec::new();
    row.write_to(&mut bytes);

    assert_eq!(bytes.len(), F16Row::encoded_len(64));
    assert_eq!(F16Row::read_from(&bytes, 64).unwrap(), row);
}

#[test]
fn a_truncated_row_is_rejected() {
    let row = F16Row::encode(&[1.0; 32]);
    let mut bytes = Vec::new();
    row.write_to(&mut bytes);
    bytes.truncate(bytes.len() - 1);
    assert!(F16Row::read_from(&bytes, 32).is_none());
}

#[test]
fn decode_into_matches_decode() {
    let v: Vec<f32> = (0..32).map(|i| i as f32 * 0.11).collect();
    let row = F16Row::encode(&v);
    let mut buf = vec![0.0; 32];
    row.decode_into(&mut buf);
    assert_eq!(buf, row.decode());
}

#[test]
fn ranking_survives_for_well_separated_vectors() {
    let query: Vec<f32> = (0..128).map(|i| (i as f32 * 0.05).cos()).collect();
    let near: Vec<f32> = query.iter().map(|x| x + 0.001).collect();
    let far: Vec<f32> = query.iter().map(|x| -x).collect();

    let dot = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
    assert!(
        dot(&query, &F16Row::encode(&near).decode()) > dot(&query, &F16Row::encode(&far).decode())
    );
}
