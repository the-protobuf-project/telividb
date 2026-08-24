use super::*;
use std::collections::BinaryHeap;

fn s(d: f32, row: u32) -> Scored {
    Scored::new(d, Ordinal::from_row(row))
}

#[test]
fn dot_and_cosine_are_negated_so_lower_is_nearer() {
    assert_eq!(to_distance(Metric::Dot, 0.9), -0.9);
    assert_eq!(to_distance(Metric::Cosine, 0.9), -0.9);
    assert_eq!(to_distance(Metric::L2, 0.9), 0.9, "already a distance");
}

#[test]
fn conversion_round_trips_for_every_metric() {
    for metric in [Metric::Dot, Metric::L2, Metric::Cosine] {
        for score in [-2.5f32, 0.0, 0.5, 17.0] {
            let back = from_distance(metric, to_distance(metric, score));
            assert_eq!(back, score, "{metric:?} did not round trip");
        }
    }
}

#[test]
fn negation_preserves_ranking_for_unnormalised_scores() {
    // Dot products are routinely outside [0, 1]; `1 - x` would be fine for
    // ordering but negation is exact and cheaper to reason about.
    let better = to_distance(Metric::Dot, 17.0);
    let worse = to_distance(Metric::Dot, 3.0);
    assert!(better < worse);
}

#[test]
fn ties_break_on_ordinal_so_ordering_is_total() {
    assert_ne!(s(1.0, 1).cmp(&s(1.0, 2)), std::cmp::Ordering::Equal);
}

#[test]
fn nearest_heap_pops_the_closest_first() {
    let mut heap = BinaryHeap::new();
    for (d, row) in [(5.0, 0), (1.0, 1), (3.0, 2)] {
        heap.push(Nearest(s(d, row)));
    }
    assert_eq!(heap.pop().unwrap().0.distance, 1.0);
    assert_eq!(heap.pop().unwrap().0.distance, 3.0);
}

#[test]
fn farthest_heap_pops_the_worst_first() {
    // This is what bounding a result set to `ef` relies on.
    let mut heap = BinaryHeap::new();
    for (d, row) in [(5.0, 0), (1.0, 1), (3.0, 2)] {
        heap.push(Farthest(s(d, row)));
    }
    assert_eq!(heap.pop().unwrap().0.distance, 5.0);
}

#[test]
fn ordering_stays_total_even_with_nan() {
    // A NaN in a partial order makes BinaryHeap loop rather than merely
    // misorder, so this must not panic or hang.
    let mut heap = BinaryHeap::new();
    heap.push(Nearest(s(f32::NAN, 0)));
    heap.push(Nearest(s(1.0, 1)));
    heap.push(Nearest(s(0.5, 2)));
    let mut popped = Vec::new();
    while let Some(x) = heap.pop() {
        popped.push(x.0.ordinal.row());
    }
    assert_eq!(popped.len(), 3);
}
