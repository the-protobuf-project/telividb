use super::*;
use crate::adapters::gpu::test_support::index_of;
use crate::adapters::{FlatIndex, MemoryStore};
use crate::ports::VectorIndex;
use telividb_core::{Dim, Metric};

const DIM: u32 = 8;

/// Distinct vectors, deliberately.
///
/// A generator that wraps produces *identical* rows, and identical rows tie.
/// CPU and GPU compute L2 differently — directly versus the
/// `‖a‖² − 2a·b + ‖b‖²` expansion — so tied scores resolve differently between
/// them, and a test comparing row identity would be asserting float
/// associativity rather than correctness.
fn store(rows: usize, metric: Metric) -> MemoryStore {
    let mut store = MemoryStore::new(Dim::new(DIM).unwrap(), metric);
    for i in 0..rows {
        let vector: Vec<f32> = (0..DIM)
            .map(|d| (i as f32) * 0.017 + (d as f32) * 0.31)
            .collect();
        store.push(&vector).unwrap();
    }
    store
}

fn queries(n: usize) -> Vec<Vec<f32>> {
    (0..n)
        .map(|q| {
            (0..DIM)
                .map(|d| ((q + d as usize) % 11) as f32 * 0.2)
                .collect()
        })
        .collect()
}

/// The property that matters: batching changes *how* scores are produced,
/// never what they are.
fn assert_matches_one_at_a_time(metric: Metric, rows: usize, n: usize, k: usize) {
    let store = store(rows, metric);
    let (index, _serial) = index_of(&store);
    let owned = queries(n);
    let refs: Vec<&[f32]> = owned.iter().map(Vec::as_slice).collect();

    let batched = index.search_batch(&store, &refs, k, None).unwrap();
    assert_eq!(batched.len(), n, "one result per query, in input order");

    for (i, query) in refs.iter().enumerate() {
        let single = index.search(&store, query, k, None).unwrap();
        let batch_rows: Vec<u32> = batched[i].iter().map(|c| c.ordinal.row()).collect();
        let single_rows: Vec<u32> = single.iter().map(|c| c.ordinal.row()).collect();
        assert_eq!(
            batch_rows, single_rows,
            "query {i} under {metric:?}: batched {batch_rows:?} vs single {single_rows:?}"
        );
    }
}

#[test]
fn a_batch_matches_answering_one_at_a_time_under_dot() {
    assert_matches_one_at_a_time(Metric::Dot, 200, 7, 5);
}

#[test]
fn a_batch_matches_answering_one_at_a_time_under_l2() {
    // L2 needs the row-norm expansion applied per query. If the batch path
    // broadcast it wrongly, every query but the first would be scored against
    // the wrong origin — and would still return plausible rows.
    assert_matches_one_at_a_time(Metric::L2, 200, 7, 5);
}

#[test]
fn a_batch_matches_answering_one_at_a_time_under_cosine() {
    assert_matches_one_at_a_time(Metric::Cosine, 200, 7, 5);
}

#[test]
fn a_batch_larger_than_one_device_call_is_split_and_still_ordered() {
    // More than MAX_BATCH, so the chunking runs. Order must survive it.
    assert_matches_one_at_a_time(Metric::Dot, 150, 70, 3);
}

#[test]
fn the_batched_result_agrees_with_the_cpu_flat_index() {
    // The end-to-end check: the device path, batched, against exhaustive CPU.
    //
    // Compared by *score* within a tolerance rather than by row. The two
    // compute L2 by different arithmetic — directly, and through the
    // `‖a‖² − 2a·b + ‖b‖²` expansion — which agree to within float error but
    // not bit-for-bit. Asserting identical rows would make this a test of
    // float associativity, and it would fail on a corpus with near-ties for a
    // reason that is not a bug.
    const TOLERANCE: f32 = 1e-3;

    let store = store(300, Metric::L2);
    let (index, _serial) = index_of(&store);
    let owned = queries(5);
    let refs: Vec<&[f32]> = owned.iter().map(Vec::as_slice).collect();

    let batched = index.search_batch(&store, &refs, 4, None).unwrap();
    for (i, query) in refs.iter().enumerate() {
        let cpu = FlatIndex::new().search(&store, query, 4, None).unwrap();
        assert_eq!(
            batched[i].len(),
            cpu.len(),
            "query {i} returned a different count"
        );

        for (gpu, exact) in batched[i].iter().zip(&cpu) {
            assert!(
                (gpu.score - exact.score).abs() < TOLERANCE,
                "query {i}: gpu scored {} where exhaustive scored {}",
                gpu.score,
                exact.score,
            );
        }
    }
}

#[test]
fn a_visibility_predicate_applies_to_every_query_in_the_batch() {
    // Invariant 15 has to hold per query, not merely for the first one.
    let store = store(200, Metric::Dot);
    let (index, _serial) = index_of(&store);
    let owned = queries(6);
    let refs: Vec<&[f32]> = owned.iter().map(Vec::as_slice).collect();

    let allowed = |o: Ordinal| o.row().is_multiple_of(3);
    let batched = index
        .search_batch(&store, &refs, 4, Some(&allowed))
        .unwrap();

    assert_eq!(batched.len(), 6);
    for result in &batched {
        assert!(!result.is_empty());
        assert!(result.iter().all(|c| c.ordinal.row().is_multiple_of(3)));
    }
}

#[test]
fn an_empty_batch_returns_nothing_rather_than_erroring() {
    let store = store(50, Metric::Dot);
    let (index, _serial) = index_of(&store);
    assert!(index.search_batch(&store, &[], 5, None).unwrap().is_empty());
}

#[test]
fn a_query_of_the_wrong_width_is_refused_for_the_whole_batch() {
    // Refused rather than partially answered: a caller cannot act on "some of
    // your queries were malformed" without being told which.
    let store = store(50, Metric::Dot);
    let (index, _serial) = index_of(&store);
    let good = vec![0.1f32; DIM as usize];
    let bad = vec![0.1f32; 3];
    let refs: Vec<&[f32]> = vec![&good, &bad];

    assert!(index.search_batch(&store, &refs, 5, None).is_err());
}
