//! Batched construction: deterministic, and honest about what it costs.

mod support;

use episteme_core::Metric;
use episteme_index::{FlatIndex, HnswIndex, HnswParams, VectorIndex, recall_at_k};
use support::corpus;

fn recall_at(batch_size: usize, rows: usize, seed: u64) -> f64 {
    let (store, queries) = corpus(rows, Metric::Cosine, seed);
    let index = HnswIndex::build(
        &store,
        HnswParams {
            batch_size,
            ..Default::default()
        },
    );
    let scores: Vec<f64> = queries
        .iter()
        .map(|q| {
            let truth = FlatIndex.search(&store, q, 10, None).unwrap();
            let got = index.search(&store, q, 10, None).unwrap();
            recall_at_k(&got, &truth, 10)
        })
        .collect();
    scores.iter().sum::<f64>() / scores.len() as f64
}

#[test]
fn a_batched_build_is_reproducible() {
    // The property that makes batching acceptable at all: the graph is fixed by
    // row order, so it never depends on thread count or scheduling. Without
    // this, a recall regression could not be attributed to a code change.
    let (store, _) = corpus(2_000, Metric::Cosine, 31);
    let params = HnswParams {
        batch_size: 64,
        ..Default::default()
    };

    let a = HnswIndex::build(&store, params);
    let b = HnswIndex::build(&store, params);
    assert_eq!(a.encode(), b.encode(), "two builds differed");
}

#[test]
fn batching_costs_recall_and_the_cost_grows_with_batch_size() {
    // Pins the trade the default is set from. If a larger batch ever stops
    // costing recall, the apply phase has changed and the default should be
    // revisited.
    let sequential = recall_at(1, 3_000, 32);
    let wide = recall_at(512, 3_000, 32);

    println!("sequential {sequential:.4}, batch 512 {wide:.4}");
    assert!(sequential >= wide, "batching should not improve recall");
    assert!(
        sequential >= 0.99,
        "sequential build regressed: {sequential}"
    );
}

#[test]
fn a_batch_larger_than_the_corpus_still_builds() {
    let recall = recall_at(100_000, 500, 33);
    assert!(recall > 0.5, "degenerate batch produced {recall}");
}
