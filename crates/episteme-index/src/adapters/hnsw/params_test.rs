use super::*;

#[test]
fn defaults_are_the_conventional_ones() {
    let p = HnswParams::default();
    assert_eq!(p.m, 16);
    assert_eq!(p.m0, 32, "layer zero gets twice the budget");
    assert!(
        p.ef_construction > p.ef_search,
        "build wider than you query"
    );
}

#[test]
fn layer_zero_gets_the_larger_budget() {
    let p = HnswParams::default();
    assert_eq!(p.max_neighbours(0), p.m0);
    assert_eq!(p.max_neighbours(1), p.m);
    assert_eq!(p.max_neighbours(7), p.m);
}

#[test]
fn ef_never_falls_below_k() {
    // Otherwise asking for more results than the candidate list holds would
    // silently return fewer, and look like a recall problem.
    let p = HnswParams {
        ef_search: 10,
        ..Default::default()
    };
    assert_eq!(p.effective_ef(5), 10);
    assert_eq!(p.effective_ef(64), 64);
}

#[test]
fn level_factor_decays_with_m() {
    let small = HnswParams {
        m: 4,
        ..Default::default()
    };
    let large = HnswParams {
        m: 64,
        ..Default::default()
    };
    assert!(
        small.level_factor() > large.level_factor(),
        "a larger m means flatter, wider layers"
    );
}

#[test]
fn batching_is_off_by_default() {
    // Measured: batching caps at ~1.26x and costs real recall, because only the
    // search half of an insert parallelises. See the field docs for the table.
    assert_eq!(HnswParams::default().batch_size, 1);
}

#[test]
fn the_seed_is_fixed_so_builds_reproduce() {
    assert_eq!(HnswParams::default().seed, HnswParams::default().seed);
}
