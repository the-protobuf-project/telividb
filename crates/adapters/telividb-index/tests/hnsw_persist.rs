//! A reopened graph must answer exactly as the one that was built.
//!
//! Rebuilding on open is minutes to hours at scale, so persistence is not an
//! optimization. What matters is that it is *lossless*: an index that reopens
//! with slightly different structure would show up as an unexplained recall
//! drift after every restart.

mod support;

use support::corpus;
use telividb_core::Metric;
use telividb_index::{HnswIndex, HnswParams, VectorIndex, recall_at_k};

#[test]
fn a_reopened_index_returns_identical_results() {
    let (store, queries) = corpus(3_000, Metric::Cosine, 21);
    let built = HnswIndex::build(&store, HnswParams::default());

    let bytes = built.encode();
    let reopened = HnswIndex::decode(&bytes, HnswParams::default()).unwrap();

    for q in &queries {
        let before = built.search(&store, q, 10, None).unwrap();
        let after = reopened.search(&store, q, 10, None).unwrap();
        assert_eq!(
            before.iter().map(|c| c.ordinal.row()).collect::<Vec<_>>(),
            after.iter().map(|c| c.ordinal.row()).collect::<Vec<_>>(),
            "a restart changed the answer"
        );
    }
}

#[test]
fn a_reopened_index_keeps_its_recall() {
    let (store, queries) = corpus(3_000, Metric::Cosine, 22);
    let built = HnswIndex::build(&store, HnswParams::default());
    let reopened = HnswIndex::decode(&built.encode(), HnswParams::default()).unwrap();

    let per_query: Vec<f64> = queries
        .iter()
        .map(|q| {
            let truth = built.search(&store, q, 10, None).unwrap();
            let after = reopened.search(&store, q, 10, None).unwrap();
            recall_at_k(&after, &truth, 10)
        })
        .collect();

    let mean = per_query.iter().sum::<f64>() / per_query.len() as f64;
    assert_eq!(mean, 1.0, "persistence must be lossless, not merely close");
}

#[test]
fn graph_structure_survives_the_round_trip() {
    let (store, _) = corpus(1_000, Metric::Cosine, 23);
    let built = HnswIndex::build(&store, HnswParams::default());
    let reopened = HnswIndex::decode(&built.encode(), HnswParams::default()).unwrap();

    assert_eq!(built.graph().len(), reopened.graph().len());
    assert_eq!(built.graph().edge_count(), reopened.graph().edge_count());
    assert_eq!(built.graph().max_level(), reopened.graph().max_level());
    assert_eq!(built.graph().entry(), reopened.graph().entry());
}

#[test]
fn encoding_is_stable_across_a_round_trip() {
    // Encode, decode, encode again — byte-identical. Without this an archive
    // could not be checksummed, since the same graph would hash differently
    // depending on how many times it had been reopened.
    let (store, _) = corpus(500, Metric::Cosine, 24);
    let built = HnswIndex::build(&store, HnswParams::default());

    let once = built.encode();
    let twice = HnswIndex::decode(&once, HnswParams::default())
        .unwrap()
        .encode();
    assert_eq!(once, twice);
}

#[test]
fn graph_size_is_independent_of_dimension() {
    // The fact that decides whether an index fits in memory, and the one that
    // surprises people: an HNSW graph costs roughly `nodes * m0 * 4` bytes and
    // does not care how wide the vectors are.
    //
    // So at 32 dimensions the graph is *larger* than the vectors it indexes,
    // and at 768 it is a small fraction of them. Sizing an index by "a
    // percentage of the data" is therefore wrong in both directions.
    let (store, _) = corpus(2_000, Metric::Cosine, 25);
    let built = HnswIndex::build(&store, HnswParams::default());
    let graph_bytes = built.encode().len();

    let params = HnswParams::default();
    let upper_bound = 2_000 * (params.m0 + 2 * params.m) * 4 + 4_096;
    assert!(
        graph_bytes < upper_bound,
        "graph {graph_bytes} exceeded the structural bound {upper_bound}"
    );

    // Narrow vectors: the graph dominates.
    let narrow_vectors = 2_000 * support::DIM * 4;
    assert!(
        graph_bytes > narrow_vectors / 2,
        "at {} dims the graph should be comparable to the vectors",
        support::DIM
    );

    // Wide vectors: the graph is a rounding error, and nothing about it changed.
    let wide_vectors = 2_000 * 768 * 4;
    assert!(
        graph_bytes < wide_vectors / 4,
        "the same graph should be small next to 768-dim vectors"
    );
}
