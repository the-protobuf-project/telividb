//! HNSW correctness, measured against exhaustive search.
//!
//! Every assertion here compares the graph to [`FlatIndex`] on the same data.
//! Speed is not asserted: a graph that answers instantly and returns the wrong
//! neighbours is broken, and recall is the only number that distinguishes the
//! two.
//!
//! Data is generated from a fixed seed so a regression is attributable to a
//! code change rather than to which vectors happened to be drawn.
//!
//! Corpora come from `support`, which is clustered by design — see that module
//! for why a uniform fixture would measure a pathology nobody encounters.

mod support;

use episteme_core::{Dim, Metric, Ordinal, VectorStore};
use episteme_index::{
    FlatIndex, HnswIndex, HnswParams, RecallReport, VectorIndex, adapters::MemoryStore, recall_at_k,
};
use support::{DIM, Rng, corpus};

/// Measure HNSW against exhaustive search over the same queries.
fn measure(
    store: &MemoryStore,
    queries: &[Vec<f32>],
    params: HnswParams,
    k: usize,
) -> RecallReport {
    let hnsw = HnswIndex::build(store, params);
    let per_query: Vec<f64> = queries
        .iter()
        .map(|q| {
            let truth = FlatIndex::new().search(store, q, k, None).unwrap();
            let approx = hnsw.search(store, q, k, None).unwrap();
            recall_at_k(&approx, &truth, k)
        })
        .collect();
    RecallReport::of(&per_query, k)
}

#[test]
fn recall_clears_the_bar_on_cosine() {
    let (store, queries) = corpus(5_000, Metric::Cosine, 11);
    let report = measure(&store, &queries, HnswParams::default(), 10);
    println!("cosine {report}");
    assert!(report.meets(0.95), "{report}");
}

#[test]
fn recall_clears_the_bar_on_l2() {
    // L2 ranks ascending. Getting the direction wrong returns the *worst* k,
    // which would show up here as recall near zero rather than as a crash.
    let (store, queries) = corpus(5_000, Metric::L2, 22);
    let report = measure(&store, &queries, HnswParams::default(), 10);
    println!("l2 {report}");
    assert!(report.meets(0.95), "{report}");
}

#[test]
fn recall_clears_the_bar_on_dot() {
    let (store, queries) = corpus(5_000, Metric::Dot, 33);
    let report = measure(&store, &queries, HnswParams::default(), 10);
    println!("dot {report}");
    assert!(report.meets(0.95), "{report}");
}

#[test]
fn no_query_collapses_even_when_the_mean_is_healthy() {
    // A mean of 0.97 can hide one query that found nothing, and that query
    // class is the one users notice.
    let (store, queries) = corpus(5_000, Metric::Cosine, 44);
    let report = measure(&store, &queries, HnswParams::default(), 10);
    assert!(report.worst >= 0.5, "a query collapsed: {report}");
}

#[test]
fn raising_ef_search_does_not_reduce_recall() {
    // The core monotonicity property: a wider candidate list cannot find less.
    let (store, queries) = corpus(3_000, Metric::Cosine, 55);
    let narrow = measure(
        &store,
        &queries,
        HnswParams {
            ef_search: 16,
            ..Default::default()
        },
        10,
    );
    let wide = measure(
        &store,
        &queries,
        HnswParams {
            ef_search: 128,
            ..Default::default()
        },
        10,
    );
    println!("narrow {narrow}\nwide   {wide}");
    assert!(
        wide.mean >= narrow.mean - 1e-9,
        "narrow {narrow}, wide {wide}"
    );
}

#[test]
fn a_build_is_reproducible() {
    let (store, queries) = corpus(2_000, Metric::Cosine, 66);
    let a = measure(&store, &queries, HnswParams::default(), 10);
    let b = measure(&store, &queries, HnswParams::default(), 10);
    assert_eq!(a.mean, b.mean, "same seed must give the same graph");
}

#[test]
fn filtering_restricts_results_without_stranding_the_graph() {
    // Traversal must still walk through excluded nodes. Refusing to would
    // strand regions and cost recall silently — so the filtered answer is
    // compared against exhaustive search under the same filter.
    let (store, queries) = corpus(3_000, Metric::Cosine, 77);
    let hnsw = HnswIndex::build(&store, HnswParams::default());
    let keep_even = |o: Ordinal| o.row().is_multiple_of(2);

    let per_query: Vec<f64> = queries
        .iter()
        .map(|q| {
            let truth = FlatIndex::new()
                .search(&store, q, 10, Some(&keep_even))
                .unwrap();
            let approx = hnsw.search(&store, q, 10, Some(&keep_even)).unwrap();
            assert!(
                approx.iter().all(|c| c.ordinal.row().is_multiple_of(2)),
                "an excluded row was returned"
            );
            recall_at_k(&approx, &truth, 10)
        })
        .collect();

    let report = RecallReport::of(&per_query, 10);
    println!("filtered {report}");
    assert!(report.meets(0.90), "{report}");
}

#[test]
fn absent_rows_are_never_returned() {
    let mut store = MemoryStore::new(Dim::new(DIM as u32).unwrap(), Metric::Cosine);
    let mut rng = Rng(88);
    for i in 0..500 {
        if i % 7 == 0 {
            store.push_absent();
        } else {
            store.push(&rng.vector()).unwrap();
        }
    }
    let hnsw = HnswIndex::build(&store, HnswParams::default());
    let hits = hnsw.search(&store, &rng.vector(), 20, None).unwrap();
    assert!(hits.iter().all(|c| store.get(c.ordinal).is_some()));
}

#[test]
fn an_empty_store_returns_nothing() {
    let store = MemoryStore::new(Dim::new(DIM as u32).unwrap(), Metric::Cosine);
    let hnsw = HnswIndex::build(&store, HnswParams::default());
    assert!(
        hnsw.search(&store, &[0.0; DIM], 10, None)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_single_row_store_returns_it() {
    let mut store = MemoryStore::new(Dim::new(DIM as u32).unwrap(), Metric::Cosine);
    let mut rng = Rng(99);
    let only = rng.vector();
    store.push(&only).unwrap();
    let hnsw = HnswIndex::build(&store, HnswParams::default());
    let hits = hnsw.search(&store, &only, 10, None).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].ordinal.row(), 0);
}

#[test]
fn dimension_mismatch_is_rejected() {
    let (store, _) = corpus(100, Metric::Cosine, 100);
    let hnsw = HnswIndex::build(&store, HnswParams::default());
    assert!(hnsw.search(&store, &[0.0; 8], 10, None).is_err());
}
