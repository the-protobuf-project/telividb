use super::*;
use crate::adapters::MemoryStore;
use telividb_core::{Dim, Metric};

fn store_of(metric: Metric, rows: &[&[f32]]) -> MemoryStore {
    let mut store = MemoryStore::new(Dim::new(2).unwrap(), metric);
    for row in rows {
        store.push(row).unwrap();
    }
    store
}

#[test]
fn dot_returns_highest_scoring_first() {
    let store = store_of(Metric::Dot, &[&[1.0, 0.0], &[0.9, 0.1], &[0.0, 1.0]]);
    let hits = FlatIndex::new()
        .search(&store, &[1.0, 0.0], 2, None)
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].ordinal.row(), 0);
    assert_eq!(hits[1].ordinal.row(), 1);
}

#[test]
fn l2_returns_lowest_distance_first() {
    let store = store_of(Metric::L2, &[&[9.0, 9.0], &[1.0, 1.0], &[5.0, 5.0]]);
    let hits = FlatIndex::new()
        .search(&store, &[0.0, 0.0], 3, None)
        .unwrap();
    let order: Vec<u32> = hits.iter().map(|h| h.ordinal.row()).collect();
    assert_eq!(
        order,
        vec![1, 2, 0],
        "nearest first regardless of metric sign"
    );
}

#[test]
fn dimension_mismatch_is_rejected() {
    let store = store_of(Metric::Dot, &[&[1.0, 0.0]]);
    let err = FlatIndex::new()
        .search(&store, &[1.0, 0.0, 0.0], 1, None)
        .unwrap_err();
    assert!(matches!(err, telividb_core::Error::DimMismatch { .. }));
}

#[test]
fn k_larger_than_corpus_returns_everything() {
    let store = store_of(Metric::Dot, &[&[1.0, 0.0], &[0.0, 1.0]]);
    assert_eq!(
        FlatIndex::new()
            .search(&store, &[1.0, 0.0], 99, None)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn k_zero_returns_nothing() {
    let store = store_of(Metric::Dot, &[&[1.0, 0.0]]);
    assert!(
        FlatIndex::new()
            .search(&store, &[1.0, 0.0], 0, None)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn absent_rows_are_skipped_not_scored() {
    let mut store = MemoryStore::new(Dim::new(2).unwrap(), Metric::Dot);
    store.push(&[1.0, 0.0]).unwrap();
    store.push_absent();
    store.push(&[0.5, 0.0]).unwrap();

    let hits = FlatIndex::new()
        .search(&store, &[1.0, 0.0], 10, None)
        .unwrap();
    assert_eq!(hits.len(), 2, "the absent row must not appear at all");
    assert!(hits.iter().all(|h| h.ordinal.row() != 1));
}

#[test]
fn filter_restricts_the_scan_and_still_fills_k() {
    // The point of pre-filtering: excluding row 0 must promote row 2 into the
    // results, not leave a hole where row 0 would have been.
    let store = store_of(Metric::Dot, &[&[1.0, 0.0], &[0.9, 0.0], &[0.8, 0.0]]);
    let deny_first = |o: telividb_core::Ordinal| o.row() != 0;

    let hits = FlatIndex::new()
        .search(&store, &[1.0, 0.0], 2, Some(&deny_first))
        .unwrap();

    assert_eq!(hits.len(), 2, "k must still be filled from visible rows");
    assert_eq!(hits[0].ordinal.row(), 1);
    assert_eq!(hits[1].ordinal.row(), 2);
}

#[test]
fn cosine_normalises_so_magnitude_does_not_rank() {
    let store = store_of(Metric::Cosine, &[&[10.0, 0.0], &[1.0, 1.0]]);
    let hits = FlatIndex::new()
        .search(&store, &[1.0, 0.0], 2, None)
        .unwrap();
    assert_eq!(hits[0].ordinal.row(), 0);
    assert!(
        (hits[0].score - 1.0).abs() < 1e-6,
        "same direction scores 1.0"
    );
}
