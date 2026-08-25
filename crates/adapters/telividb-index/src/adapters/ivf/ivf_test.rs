use super::*;
use crate::adapters::{FlatIndex, MemoryStore};
use telividb_core::{Dim, Metric};

/// A corpus of `rows` deterministic vectors in a few loose clusters.
fn store(rows: usize, dim: usize, metric: Metric) -> MemoryStore {
    let mut store = MemoryStore::new(Dim::new(dim as u32).unwrap(), metric);
    for i in 0..rows {
        let cluster = (i % 8) as f32;
        let vector: Vec<f32> = (0..dim)
            .map(|d| cluster + ((i * 31 + d * 17) % 100) as f32 * 0.003)
            .collect();
        store.push(&vector).unwrap();
    }
    store
}

fn query(dim: usize) -> Vec<f32> {
    (0..dim).map(|d| 3.0 + (d % 7) as f32 * 0.01).collect()
}

#[test]
fn every_present_row_is_assigned_to_exactly_one_list() {
    // A row in no list is unreachable; a row in two would be returned twice.
    let store = store(500, 8, Metric::L2);
    let index = IvfFlatIndex::build(&store, IvfParams::for_rows(500)).unwrap();

    assert_eq!(index.list_sizes().iter().sum::<usize>(), 500);
}

#[test]
fn probing_every_list_matches_exhaustive_search_exactly() {
    // With nprobe = nlist, IVF scans everything — so any disagreement with the
    // flat index is a scoring or selection bug, not an approximation.
    let store = store(400, 8, Metric::L2);
    let params = IvfParams::for_rows(400);
    let index = IvfFlatIndex::build(&store, params)
        .unwrap()
        .with_nprobe(params.nlist);

    let ivf = index.search(&store, &query(8), 10, None).unwrap();
    let flat = FlatIndex::new()
        .search(&store, &query(8), 10, None)
        .unwrap();

    let ivf_rows: Vec<u32> = ivf.iter().map(|c| c.ordinal.row()).collect();
    let flat_rows: Vec<u32> = flat.iter().map(|c| c.ordinal.row()).collect();
    assert_eq!(
        ivf_rows, flat_rows,
        "ivf {ivf_rows:?} vs flat {flat_rows:?}"
    );
}

#[test]
fn raising_nprobe_never_lowers_recall() {
    // The dial has to be monotonic, or it is not a dial. A non-monotonic
    // nprobe would make the recall curve meaningless.
    let store = store(600, 8, Metric::L2);
    let params = IvfParams::for_rows(600);
    let truth: Vec<u32> = FlatIndex::new()
        .search(&store, &query(8), 10, None)
        .unwrap()
        .iter()
        .map(|c| c.ordinal.row())
        .collect();

    let mut previous = 0usize;
    for nprobe in [1usize, 2, 4, 8, params.nlist] {
        let index = IvfFlatIndex::build(&store, params)
            .unwrap()
            .with_nprobe(nprobe);
        let found = index.search(&store, &query(8), 10, None).unwrap();
        let hits = found
            .iter()
            .filter(|c| truth.contains(&c.ordinal.row()))
            .count();
        assert!(
            hits >= previous,
            "nprobe {nprobe} recalled {hits}, fewer than the {previous} before it"
        );
        previous = hits;
    }
    assert_eq!(previous, truth.len(), "probing every list should be exact");
}

#[test]
fn a_visibility_predicate_is_applied_during_the_scan() {
    // Invariant 15: filtering after selection leaks how many rows were hidden
    // and where they ranked.
    let store = store(300, 8, Metric::L2);
    let params = IvfParams::for_rows(300);
    let index = IvfFlatIndex::build(&store, params)
        .unwrap()
        .with_nprobe(params.nlist);

    let allowed = |o: telividb_core::Ordinal| o.row().is_multiple_of(2);
    let found = index.search(&store, &query(8), 10, Some(&allowed)).unwrap();

    assert!(!found.is_empty());
    assert!(
        found.iter().all(|c| c.ordinal.row().is_multiple_of(2)),
        "got {found:?}"
    );
}

#[test]
fn an_empty_store_yields_an_index_that_matches_nothing() {
    // A field no point populates is ordinary, not an error.
    let store = MemoryStore::new(Dim::new(8).unwrap(), Metric::L2);
    let index = IvfFlatIndex::build(&store, IvfParams::default()).unwrap();

    assert!(
        index
            .search(&store, &query(8), 10, None)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_query_of_the_wrong_width_is_refused() {
    let store = store(100, 8, Metric::L2);
    let index = IvfFlatIndex::build(&store, IvfParams::for_rows(100)).unwrap();

    assert!(index.search(&store, &[1.0, 2.0], 10, None).is_err());
}

#[test]
fn more_clusters_than_rows_is_clamped_rather_than_leaving_empty_lists() {
    // k-means would otherwise reseed the surplus centroids onto arbitrary
    // points, and those lists would attract rows for no reason.
    let store = store(10, 4, Metric::L2);
    let index = IvfFlatIndex::build(
        &store,
        IvfParams {
            nlist: 500,
            ..IvfParams::default()
        },
    )
    .unwrap();

    assert!(index.list_sizes().len() <= 10);
    assert_eq!(index.list_sizes().iter().sum::<usize>(), 10);
}

#[test]
fn cosine_probes_by_the_collections_own_metric() {
    // Probing under a different measure than the search picks the wrong lists
    // confidently. With every list probed the result must still be exact.
    let store = store(300, 8, Metric::Cosine);
    let params = IvfParams::for_rows(300);
    let index = IvfFlatIndex::build(&store, params)
        .unwrap()
        .with_nprobe(params.nlist);

    let ivf = index.search(&store, &query(8), 5, None).unwrap();
    let flat = FlatIndex::new().search(&store, &query(8), 5, None).unwrap();
    assert_eq!(
        ivf.iter().map(|c| c.ordinal.row()).collect::<Vec<_>>(),
        flat.iter().map(|c| c.ordinal.row()).collect::<Vec<_>>(),
    );
}
