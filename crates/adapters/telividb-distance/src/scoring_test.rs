use super::*;
use crate::scoring::Scorer;

#[test]
fn dot_and_cosine_score_identically() {
    // Cosine is dot over normalised vectors; the metric does not re-normalise,
    // because ingest already did. If these ever diverge, something started
    // normalising twice.
    let q = [0.6f32, 0.8];
    let c = [1.0f32, 0.0];
    assert_eq!(Metric::Dot.score(&q, &c), Metric::Cosine.score(&q, &c));
}

#[test]
fn l2_scores_distance_where_dot_scores_similarity() {
    // Identical vectors: distance 0, similarity 1. The two run in opposite
    // directions, which is exactly why nothing may assume a sort order.
    let v = [1.0f32, 0.0];
    assert!(Metric::L2.score(&v, &v).abs() < 1e-6);
    assert!((Metric::Dot.score(&v, &v) - 1.0).abs() < 1e-6);
}

#[test]
fn each_metric_agrees_with_its_direction_flag() {
    // The flag is what every selection path sorts by, so a metric whose scores
    // ran the other way would rank everything backwards while erroring nowhere.
    let query = [1.0f32, 0.0];
    let near = [0.9f32, 0.1];
    let far = [0.0f32, 1.0];

    for metric in [Metric::Dot, Metric::Cosine, Metric::L2] {
        let near_score = metric.score(&query, &near);
        let far_score = metric.score(&query, &far);
        let nearer_wins = match metric.higher_is_nearer() {
            true => near_score > far_score,
            false => near_score < far_score,
        };
        assert!(nearer_wins, "{metric:?} ranked the far vector as nearer");
    }
}
