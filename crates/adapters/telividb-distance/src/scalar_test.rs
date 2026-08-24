use super::*;

#[test]
fn dot_of_orthogonal_is_zero() {
    assert_eq!(dot(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
}

#[test]
fn l2_of_identical_is_zero() {
    assert_eq!(l2_squared(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]), 0.0);
}

#[test]
fn normalize_yields_unit_length() {
    let mut v = [3.0, 4.0];
    normalize(&mut v);
    assert!((dot(&v, &v).sqrt() - 1.0).abs() < 1e-6);
}

#[test]
fn normalize_leaves_zero_vector_alone() {
    let mut v = [0.0, 0.0];
    normalize(&mut v);
    assert_eq!(v, [0.0, 0.0], "must not produce NaN");
}

#[test]
fn normalized_dot_matches_cosine() {
    let (mut a, mut b) = ([1.0, 1.0], [1.0, 0.0]);
    normalize(&mut a);
    normalize(&mut b);
    // cos(45°)
    assert!((dot(&a, &b) - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
}
