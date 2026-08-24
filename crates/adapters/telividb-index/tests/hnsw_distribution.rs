//! Why the recall fixtures are clustered.
//!
//! Kept apart from `hnsw_recall` because it asks a different question: not "is
//! the index good" but "is the fixture measuring anything real". It is the
//! justification for every other recall number in the suite.

mod support;

use support::{CLUSTERS, Rng};
use telividb_core::{Dim, Metric};
use telividb_index::{
    FlatIndex, HnswIndex, HnswParams, RecallReport, VectorIndex, adapters::MemoryStore, recall_at_k,
};

fn score(store: &MemoryStore, queries: &[Vec<f32>]) -> RecallReport {
    let hnsw = HnswIndex::build(store, HnswParams::default());
    let per_query: Vec<f64> = queries
        .iter()
        .map(|q| {
            let truth = FlatIndex::new().search(store, q, 10, None).unwrap();
            let approx = hnsw.search(store, q, 10, None).unwrap();
            recall_at_k(&approx, &truth, 10)
        })
        .collect();
    RecallReport::of(&per_query, 10)
}

#[test]
fn uniform_data_is_adversarial_and_that_is_the_data_not_the_index() {
    // Pins down why every other test here uses clustered data.
    //
    // Concentration of measure only bites at realistic embedding widths, so
    // this test uses 128 dimensions rather than the 32 the fast tests share.
    // At 32 dimensions uniform data still scores 1.0, which is exactly the trap:
    // a low-dimensional fixture makes an index look fine and says nothing about
    // how it behaves on real vectors.
    //
    // If the gap ever closes, either the generator stopped being uniform or the
    // dimension dropped.
    const WIDE: usize = 128;
    let rows = 4_000;

    let mut rng = Rng(123);
    let mut uniform_store = MemoryStore::new(Dim::new(WIDE as u32).unwrap(), Metric::Cosine);
    for _ in 0..rows {
        uniform_store.push(&rng.vector_of(WIDE)).unwrap();
    }
    let uniform_queries: Vec<Vec<f32>> = (0..20).map(|_| rng.vector_of(WIDE)).collect();

    let centres: Vec<Vec<f32>> = (0..CLUSTERS).map(|_| rng.vector_of(WIDE)).collect();
    let mut clustered_store = MemoryStore::new(Dim::new(WIDE as u32).unwrap(), Metric::Cosine);
    for row in 0..rows {
        let v = rng.near(&centres[row % CLUSTERS]);
        clustered_store.push(&v).unwrap();
    }
    let clustered_queries: Vec<Vec<f32>> =
        (0..20).map(|i| rng.near(&centres[i % CLUSTERS])).collect();

    let uniform = score(&uniform_store, &uniform_queries);
    let clustered = score(&clustered_store, &clustered_queries);

    println!("uniform   {uniform}");
    println!("clustered {clustered}");
    assert!(
        clustered.mean > uniform.mean + 0.05,
        "clustered {clustered} should clearly beat uniform {uniform}"
    );
}
