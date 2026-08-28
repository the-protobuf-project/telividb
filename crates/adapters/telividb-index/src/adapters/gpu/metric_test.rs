//! The metric conversions, against their definitions.

use super::ScoreFromDot;
use telividb_core::Metric;

#[test]
fn dot_and_cosine_are_the_product_itself() {
    assert_eq!(Metric::Dot.score_of(0.75, 9.0, 4.0), 0.75);
    assert_eq!(Metric::Cosine.score_of(0.75, 9.0, 4.0), 0.75);
}

#[test]
fn l2_expands_to_the_squared_distance() {
    // a = [1, 2], b = [4, 6]: ‖a−b‖² = 9 + 16 = 25.
    let (a, b) = ([1.0f32, 2.0], [4.0f32, 6.0]);
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let query_norm: f32 = a.iter().map(|x| x * x).sum();
    let row_norm: f32 = b.iter().map(|x| x * x).sum();

    assert_eq!(Metric::L2.score_of(dot, row_norm, query_norm), 25.0);
}

#[test]
fn l2_of_a_vector_with_itself_is_zero() {
    let v = [0.3f32, -1.4, 2.0];
    let norm: f32 = v.iter().map(|x| x * x).sum();
    // Not exactly zero in floating point, but it must be indistinguishable
    // from it — an identical vector ranking behind a merely-similar one would
    // be a visible ordering bug.
    assert!(Metric::L2.score_of(norm, norm, norm).abs() < 1e-5);
}
