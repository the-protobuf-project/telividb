//! Decay and ranking, under both metric directions.

use super::Expansion;

#[test]
fn decay_weakens_a_neighbour_under_either_metric() {
    // The property that has to hold whichever way the metric points: a hop
    // makes a result *worse*, never better.
    let higher = Expansion::default().decaying(0.5);
    assert!(higher.decayed(1.0, 1) < 1.0, "higher-is-nearer must shrink");

    let lower = Expansion::default().decaying(0.5).lower_is_nearer();
    assert!(
        lower.decayed(1.0, 1) > 1.0,
        "lower-is-nearer must grow — multiplying an L2 distance by 0.5 would \
         make a neighbour look nearer than the seed that found it"
    );
}

#[test]
fn decay_compounds_with_distance() {
    let e = Expansion::default().decaying(0.5);
    assert_eq!(e.decayed(1.0, 1), 0.5);
    assert_eq!(e.decayed(1.0, 2), 0.25);
}

#[test]
fn ranking_follows_the_metric_direction() {
    use std::cmp::Ordering;
    assert_eq!(
        Expansion::default().rank(0.9, 0.1),
        Ordering::Less,
        "0.9 ranks first"
    );
    assert_eq!(
        Expansion::default().lower_is_nearer().rank(0.9, 0.1),
        Ordering::Greater,
        "under L2 the smaller distance ranks first"
    );
}

#[test]
fn a_zero_decay_does_not_divide_by_zero() {
    // Degenerate but reachable through the builder, and an infinity here would
    // propagate into every comparison.
    let e = Expansion::default().decaying(0.0).lower_is_nearer();
    assert!(e.decayed(1.0, 1).is_finite());
}
