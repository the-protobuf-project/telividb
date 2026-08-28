use super::*;

#[test]
fn dot_and_l2_agree_with_their_definitions() {
    let a = [1.0f32, 2.0, 3.0];
    let b = [4.0f32, 5.0, 6.0];
    assert!((a.dot(&b) - 32.0).abs() < 1e-6);
    // (1-4)² + (2-5)² + (3-6)² = 27
    assert!((a.l2_squared(&b) - 27.0).abs() < 1e-6);
}

#[test]
fn l2_is_squared_rather_than_rooted() {
    // The whole codebase treats Metric::L2 as squared distance. If this ever
    // returned the root, every stored score would change meaning silently.
    let a = [3.0f32, 4.0];
    let origin = [0.0f32, 0.0];
    assert!(
        (a.l2_squared(&origin) - 25.0).abs() < 1e-6,
        "should be 25, not 5"
    );
    assert!((a.norm() - 5.0).abs() < 1e-6, "norm is rooted");
}

#[test]
fn normalizing_produces_unit_length() {
    let v = [3.0f32, 4.0];
    let unit = v.normalized();
    assert!((unit.norm() - 1.0).abs() < 1e-6, "got {unit:?}");
}

#[test]
fn a_zero_vector_survives_normalisation_rather_than_becoming_nan() {
    // Dividing by a zero norm would poison every later comparison, and a NaN
    // makes an ordering inconsistent rather than merely wrong.
    let zero = [0.0f32; 4];
    let out = zero.normalized();
    assert!(out.iter().all(|v| v.is_finite()), "got {out:?}");
    assert_eq!(out, vec![0.0; 4]);
}

#[test]
fn in_place_normalisation_matches_the_copying_form() {
    let original = [1.0f32, 2.0, 2.0];
    let mut owned = original;
    owned.normalize();
    assert_eq!(owned.to_vec(), original.normalized());
}

#[test]
fn a_shorter_operand_stops_early_rather_than_panicking() {
    // Widths are checked at the API boundary where a mismatch is still
    // attributable; this must not be the thing that panics.
    let a = [1.0f32, 2.0, 3.0];
    let short = [1.0f32, 1.0];
    assert!(a.dot(&short).is_finite());
    assert!(a.l2_squared(&short).is_finite());
}
