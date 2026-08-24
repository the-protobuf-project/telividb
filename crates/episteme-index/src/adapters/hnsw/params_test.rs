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
fn batching_is_on_by_default() {
    // It was off because the measured recall curve fell away with batch size.
    // That curve was a bug — the first batch searched an empty graph and was
    // orphaned wholesale — and with it fixed, recall is flat across batch
    // sizes while the build gets about 1.25x faster. See the field docs for the
    // table and `hnsw_parallel` for the floor that keeps it honest.
    assert_eq!(HnswParams::default().batch_size, 128);
}

#[test]
fn the_seed_is_fixed_so_builds_reproduce() {
    assert_eq!(HnswParams::default().seed, HnswParams::default().seed);
}
