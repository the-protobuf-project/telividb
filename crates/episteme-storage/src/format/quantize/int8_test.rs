use super::*;

fn max_abs_error(original: &[f32], decoded: &[f32]) -> f32 {
    original
        .iter()
        .zip(decoded)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max)
}

#[test]
fn round_trip_stays_within_half_a_step() {
    // The bound the codec promises. Anything worse means the mapping is wrong,
    // not merely lossy.
    let v: Vec<f32> = (0..768).map(|i| (i as f32 * 0.017).sin()).collect();
    let row = Int8Row::encode(&v);
    let back = row.decode();
    assert!(
        max_abs_error(&v, &back) <= row.max_error() * 1.001,
        "error {} exceeded bound {}",
        max_abs_error(&v, &back),
        row.max_error()
    );
}

#[test]
fn endpoints_are_exact() {
    let v = vec![-3.0, 0.0, 5.0];
    let back = Int8Row::encode(&v).decode();
    assert!((back[0] - (-3.0)).abs() < 1e-4, "min must map exactly");
    assert!((back[2] - 5.0).abs() < 1e-4, "max must map exactly");
}

#[test]
fn a_constant_row_does_not_divide_by_zero() {
    // Zero vectors and padding rows are constant, and they are common.
    let row = Int8Row::encode(&[2.5; 16]);
    assert_eq!(row.scale, 0.0);
    assert!(row.decode().iter().all(|&x| (x - 2.5).abs() < 1e-6));
}

#[test]
fn an_all_zero_row_decodes_to_zero() {
    assert!(
        Int8Row::encode(&[0.0; 8])
            .decode()
            .iter()
            .all(|&x| x == 0.0)
    );
}

#[test]
fn non_finite_input_does_not_poison_the_scale() {
    // A NaN reaching here means validation upstream failed, but the codec must
    // still produce something finite rather than propagating the poison.
    let row = Int8Row::encode(&[1.0, f32::NAN, 2.0]);
    assert!(row.scale.is_finite() && row.offset.is_finite());
    assert!(row.decode().iter().all(|x| x.is_finite()));
}

#[test]
fn rounding_is_unbiased_rather_than_truncating() {
    // Truncation would pull every component toward the minimum, shifting the
    // whole vector and showing up as a systematic ranking error.
    let v: Vec<f32> = (0..256).map(|i| i as f32 / 255.0).collect();
    let back = Int8Row::encode(&v).decode();
    let mean_error: f32 = v.iter().zip(&back).map(|(a, b)| b - a).sum::<f32>() / v.len() as f32;
    assert!(mean_error.abs() < 1e-4, "biased by {mean_error}");
}

#[test]
fn compression_ratio_is_about_four_times() {
    assert_eq!(Int8Row::encoded_len(768), 776);
    let ratio = (768 * 4) as f32 / 776.0;
    assert!(ratio > 3.9 && ratio < 4.0, "ratio {ratio}");
}

#[test]
fn serialization_round_trips() {
    let v: Vec<f32> = (0..64).map(|i| i as f32 * 0.3 - 5.0).collect();
    let row = Int8Row::encode(&v);
    let mut bytes = Vec::new();
    row.write_to(&mut bytes);

    assert_eq!(bytes.len(), Int8Row::encoded_len(64));
    assert_eq!(Int8Row::read_from(&bytes, 64).unwrap(), row);
}

#[test]
fn a_truncated_row_is_rejected_rather_than_misread() {
    let row = Int8Row::encode(&[1.0; 32]);
    let mut bytes = Vec::new();
    row.write_to(&mut bytes);
    bytes.truncate(bytes.len() - 3);
    assert!(Int8Row::read_from(&bytes, 32).is_none());
}

#[test]
fn decode_into_matches_decode() {
    let v: Vec<f32> = (0..32).map(|i| i as f32 * 0.11).collect();
    let row = Int8Row::encode(&v);
    let mut buf = vec![0.0; 32];
    row.decode_into(&mut buf);
    assert_eq!(buf, row.decode());
}

#[test]
fn ranking_survives_quantization_for_well_separated_vectors() {
    // The property that actually matters: quantization may perturb scores, but
    // it must not reorder candidates that were clearly apart.
    let query: Vec<f32> = (0..128).map(|i| (i as f32 * 0.05).cos()).collect();
    let near: Vec<f32> = query.iter().map(|x| x + 0.01).collect();
    let far: Vec<f32> = query.iter().map(|x| -x).collect();

    let near_q = Int8Row::encode(&near).decode();
    let far_q = Int8Row::encode(&far).decode();

    let dot = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
    assert!(
        dot(&query, &near_q) > dot(&query, &far_q),
        "quantization reordered clearly separated candidates"
    );
}
