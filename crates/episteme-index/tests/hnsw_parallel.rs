//! Batched construction: deterministic, and honest about what it costs.

mod support;

use episteme_core::{Dim, Metric, Ordinal};
use episteme_index::adapters::MemoryStore;
use episteme_index::{FlatIndex, HnswIndex, HnswParams, VectorIndex, VectorStore, recall_at_k};
use support::{DIM, Rng, corpus};

/// Rows, chosen so several batch sizes divide it unevenly.
const ROWS: usize = 3_000;

fn index_at(batch_size: usize, rows: usize, seed: u64) -> (MemoryStore, Vec<Vec<f32>>, HnswIndex) {
    let (store, queries) = corpus(rows, Metric::Cosine, seed);
    let index = HnswIndex::build(
        &store,
        HnswParams {
            batch_size,
            ..Default::default()
        },
    );
    (store, queries, index)
}

fn recall_at(batch_size: usize, rows: usize, seed: u64) -> f64 {
    let (store, queries, index) = index_at(batch_size, rows, seed);
    let scores: Vec<f64> = queries
        .iter()
        .map(|q| {
            let truth = FlatIndex::new().search(&store, q, 10, None).unwrap();
            let got = index.search(&store, q, 10, None).unwrap();
            recall_at_k(&got, &truth, 10)
        })
        .collect();
    scores.iter().sum::<f64>() / scores.len() as f64
}

/// Rows that have a vector but no layer-0 edges.
///
/// The direct symptom of a node inserted against a graph it could not search:
/// it is pushed, never linked, and nothing can reach it afterwards because
/// nothing knows it is there.
fn layer0_orphans(store: &MemoryStore, index: &HnswIndex) -> usize {
    let graph = index.graph();
    (0..store.len())
        .filter(|&row| store.get(Ordinal::from_row(row as u32)).is_some())
        .filter(|&row| graph.neighbours(row as u32, 0).is_empty())
        .count()
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
fn no_batch_size_orphans_a_present_row() {
    // The regression this file exists for. Every node in batch zero used to
    // search an empty snapshot, find nothing, and be pushed with no edges —
    // orphaning exactly `batch_size - 1` rows at every batch size, and nothing
    // here measured it.
    for batch_size in [1, 32, 64, 256, 512] {
        let (store, _, index) = index_at(batch_size, ROWS, 32);
        let orphans = layer0_orphans(&store, &index);
        assert_eq!(
            orphans, 0,
            "batch_size {batch_size} left {orphans} present rows unreachable"
        );
    }
}

#[test]
fn every_batch_size_holds_an_absolute_recall_floor() {
    // A relative assertion — `sequential >= wide` — passes at any absolute
    // quality, and did: 0.5675 at batch 512 satisfied it while more than
    // two-fifths of the true neighbours were missing.
    const FLOOR: f64 = 0.97;
    for batch_size in [1, 32, 64, 256, 512] {
        let recall = recall_at(batch_size, ROWS, 32);
        println!("batch_size {batch_size:>4}: recall@10 {recall:.4}");
        assert!(
            recall >= FLOOR,
            "batch_size {batch_size} scored {recall:.4}, below the {FLOOR} floor"
        );
    }
}

#[test]
fn batching_stays_close_to_a_sequential_build() {
    // Pins the trade the default is set from. Batching may still cost a little
    // — nodes within one batch cannot link to each other — but it must be a
    // rounding error rather than a cliff.
    let sequential = recall_at(1, ROWS, 32);
    let wide = recall_at(512, ROWS, 32);

    println!("sequential {sequential:.4}, batch 512 {wide:.4}");
    assert!(
        sequential >= 0.99,
        "sequential build regressed: {sequential}"
    );
    assert!(
        sequential - wide < 0.02,
        "batching cost {:.4} recall, which is a cliff rather than a trade",
        sequential - wide
    );
}

#[test]
fn a_batch_larger_than_the_corpus_takes_the_batched_path() {
    // `build` only batches when `store.len() > params.batch_size`, so a batch
    // of 100_000 against 500 rows quietly ran the *sequential* builder — this
    // test never exercised batching at all. A batch just under the row count
    // does, and is the degenerate case worth covering: one full batch, then a
    // remainder of one.
    let rows = 500;
    let (store, _, index) = index_at(rows - 1, rows, 33);
    assert_eq!(
        layer0_orphans(&store, &index),
        0,
        "a single-batch build orphaned rows"
    );
    let recall = recall_at(rows - 1, rows, 33);
    assert!(recall > 0.97, "degenerate batch produced {recall}");
}

#[test]
fn an_absent_row_at_the_head_does_not_strand_the_rows_after_it() {
    // An absent row became the entry point unconditionally, and an entry that
    // cannot be scored makes `distance_to` return `None` — so every insert
    // after it bailed out before linking anything, until some later node drew
    // a higher level and took over.
    for absent_prefix in [1, 5, 20, 100] {
        let mut rng = Rng(77);
        let mut store = MemoryStore::new(Dim::new(DIM as u32).unwrap(), Metric::Cosine);
        for _ in 0..absent_prefix {
            store.push_absent();
        }
        let centre = rng.vector();
        for _ in 0..2_000 {
            let v = rng.near(&centre);
            store.push(&v).unwrap();
        }

        let index = HnswIndex::build(&store, HnswParams::default());
        let orphans = layer0_orphans(&store, &index);
        assert_eq!(
            orphans, 0,
            "an absent prefix of {absent_prefix} stranded {orphans} present rows"
        );
        assert!(
            index
                .graph()
                .entry()
                .is_some_and(|e| store.get(e).is_some()),
            "the entry point is an absent row"
        );
    }
}

#[test]
fn a_batch_that_raises_the_top_layer_still_links_its_own_layer_zero() {
    // `apply` used to re-derive each candidate list's layer from its position
    // and `graph.max_level()` — read *after* earlier nodes in the same batch
    // had been applied. A batch-mate that raised the top layer shifted every
    // index, so a node's layer-0 candidates were written to layer 1 and the
    // node was unreachable at the only layer a search finishes on.
    //
    // Level assignment is seeded, so this reproduces: it needs a batch in which
    // some node draws a new maximum level before a later node in the same batch
    // is applied. Sweeping seeds and batch sizes finds those reliably.
    for seed in [7u64, 77, 777] {
        for batch_size in [16usize, 64, 128] {
            let (store, _) = corpus(2_000, Metric::Cosine, seed);
            let index = HnswIndex::build(
                &store,
                HnswParams {
                    batch_size,
                    seed,
                    ..Default::default()
                },
            );
            assert_eq!(
                layer0_orphans(&store, &index),
                0,
                "seed {seed}, batch_size {batch_size} left rows off layer zero"
            );
        }
    }
}
