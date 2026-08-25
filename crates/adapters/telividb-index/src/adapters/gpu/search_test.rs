use super::*;
use crate::adapters::MemoryStore;
use crate::ports::VectorIndex;
use crate::adapters::gpu::gguf::{load_corpus, write_corpus};
use candle_core::Device;
use telividb_core::Dim;

const DIM: u32 = 4;

fn corpus_of(store: &MemoryStore) -> Corpus {
    let mut buffer = std::io::Cursor::new(Vec::new());
    write_corpus(store, &mut buffer).unwrap();
    buffer.set_position(0);
    load_corpus(&mut buffer, &Device::Cpu).unwrap()
}

/// Three unit-ish rows whose dot products against `[1,0,0,0]` are 1, 2, 3 —
/// so the expected ranking is unambiguous.
fn ranked_store() -> MemoryStore {
    let mut store = MemoryStore::new(Dim::new(DIM).unwrap(), Metric::Dot);
    store.push(&[1.0, 0.0, 0.0, 0.0]).unwrap();
    store.push(&[2.0, 0.0, 0.0, 0.0]).unwrap();
    store.push(&[3.0, 0.0, 0.0, 0.0]).unwrap();
    store
}

fn query() -> Vec<f32> {
    vec![1.0, 0.0, 0.0, 0.0]
}

#[test]
fn results_come_back_best_first() {
    let corpus = corpus_of(&ranked_store());
    let hits = search(&corpus, &query(), 3, None).unwrap();

    let rows: Vec<u32> = hits.iter().map(|c| c.ordinal.row()).collect();
    assert_eq!(rows, vec![2, 1, 0], "dot: higher is nearer");
    assert_eq!(hits[0].score, 3.0);
}

#[test]
fn k_truncates_without_reordering() {
    let corpus = corpus_of(&ranked_store());
    let hits = search(&corpus, &query(), 2, None).unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].ordinal.row(), 2);
}

#[test]
fn k_of_zero_returns_nothing() {
    let corpus = corpus_of(&ranked_store());
    assert!(search(&corpus, &query(), 0, None).unwrap().is_empty());
}

#[test]
fn a_wrong_width_query_is_refused() {
    let corpus = corpus_of(&ranked_store());
    assert!(search(&corpus, &[1.0, 0.0], 3, None).is_err());
}

#[test]
fn an_absent_row_never_scores() {
    // Zeros against a dot product are a real score, not a neutral one — so an
    // absent row would rank above any negatively-scoring present row if the
    // presence mask were not consulted.
    let mut store = MemoryStore::new(Dim::new(DIM).unwrap(), Metric::Dot);
    store.push(&[-1.0, 0.0, 0.0, 0.0]).unwrap();
    store.push_absent();
    let corpus = corpus_of(&store);

    let hits = search(&corpus, &query(), 5, None).unwrap();
    assert_eq!(hits.len(), 1, "only the present row is a candidate");
    assert_eq!(hits[0].ordinal.row(), 0);
}

#[test]
fn a_hidden_row_is_dropped_before_selection_so_k_stays_full() {
    // The property invariant 15 actually requires: hiding the best row must
    // still return `k` results drawn from what remains — never `k` minus the
    // hidden ones, which would disclose how many were hidden.
    let corpus = corpus_of(&ranked_store());
    let hide_best = |o: Ordinal| o.row() != 2;

    let hits = search(&corpus, &query(), 2, Some(&hide_best)).unwrap();
    assert_eq!(hits.len(), 2, "still a full k");
    let rows: Vec<u32> = hits.iter().map(|c| c.ordinal.row()).collect();
    assert_eq!(rows, vec![1, 0]);
    assert!(!rows.contains(&2));
}

#[test]
fn searching_an_empty_corpus_returns_nothing_rather_than_aborting() {
    // Regression: candle's GGUF reader dereferences a null pointer on a
    // zero-element tensor, which aborts the process rather than erroring — so
    // an empty field, which is entirely ordinary, used to kill the server.
    let store = MemoryStore::new(Dim::new(DIM).unwrap(), Metric::Dot);
    let corpus = corpus_of(&store);
    assert!(search(&corpus, &query(), 5, None).unwrap().is_empty());
}

#[test]
fn l2_ranks_by_distance_with_the_nearest_first() {
    // The expansion `‖a‖² − 2a·b + ‖b‖²` must rank *lower* as nearer, which
    // is the opposite of dot. Getting the sign wrong returns the farthest
    // vectors confidently labelled as the nearest.
    let mut store = MemoryStore::new(Dim::new(DIM).unwrap(), Metric::L2);
    store.push(&[1.0, 0.0, 0.0, 0.0]).unwrap(); // distance 0 from the query
    store.push(&[0.0, 1.0, 0.0, 0.0]).unwrap(); // distance 2
    store.push(&[5.0, 0.0, 0.0, 0.0]).unwrap(); // distance 16
    let corpus = corpus_of(&store);

    let found = search(&corpus, &query(), 3, None).unwrap();
    assert_eq!(found[0].ordinal.row(), 0, "got {found:?}");
    assert_eq!(found[1].ordinal.row(), 1, "got {found:?}");
    assert_eq!(found[2].ordinal.row(), 2, "got {found:?}");
}

#[test]
fn l2_scores_are_squared_distances_not_merely_an_ordering() {
    // The scores are handed to the caller, so a value that ranks correctly
    // while being numerically wrong is the kind of thing that is trusted
    // until someone thresholds on it.
    let mut store = MemoryStore::new(Dim::new(DIM).unwrap(), Metric::L2);
    store.push(&[1.0, 0.0, 0.0, 0.0]).unwrap();
    store.push(&[0.0, 1.0, 0.0, 0.0]).unwrap();
    let corpus = corpus_of(&store);

    let found = search(&corpus, &query(), 2, None).unwrap();
    // query is [1,0,0,0]: distance² to itself is 0, to [0,1,0,0] is 2.
    assert!(found[0].score.abs() < 1e-4, "got {}", found[0].score);
    assert!((found[1].score - 2.0).abs() < 1e-4, "got {}", found[1].score);
}

#[test]
fn l2_agrees_with_the_cpu_flat_index() {
    // The GPU index's correctness standard is equality with exhaustive CPU
    // search, not a recall threshold — so a divergence here is a bug rather
    // than an acceptable approximation.
    let mut store = MemoryStore::new(Dim::new(DIM).unwrap(), Metric::L2);
    for i in 0..64u32 {
        let f = i as f32;
        store.push(&[f * 0.1, 1.0 - f * 0.01, f * 0.02, 0.5]).unwrap();
    }
    let corpus = corpus_of(&store);

    let gpu = search(&corpus, &query(), 8, None).unwrap();
    let cpu = crate::adapters::FlatIndex::new()
        .search(&store, &query(), 8, None)
        .unwrap();

    let gpu_rows: Vec<u32> = gpu.iter().map(|c| c.ordinal.row()).collect();
    let cpu_rows: Vec<u32> = cpu.iter().map(|c| c.ordinal.row()).collect();
    assert_eq!(gpu_rows, cpu_rows, "gpu {gpu_rows:?} vs cpu {cpu_rows:?}");
}
