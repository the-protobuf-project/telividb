use super::*;
use crate::adapters::ivf::{IvfParams, IvfPqIndex};
use crate::adapters::{FlatIndex, MemoryStore};
use crate::ports::VectorIndex;
use telividb_core::Dim;
use telividb_core::Metric;
use telividb_distance::pq::PqParams;

/// A corpus with real cluster structure, so quantization has something to
/// preserve. Random noise would make any codebook look equally good.
///
/// Every corpus here clears 256 rows: a codebook needs at least one training
/// vector per centroid, and `PqCodebook::train` refuses fewer rather than
/// producing a degenerate codebook that encodes everything identically.
fn store(rows: usize, dim: usize, metric: Metric) -> MemoryStore {
    let mut store = MemoryStore::new(Dim::new(dim as u32).unwrap(), metric);
    for i in 0..rows {
        let centre = (i % 16) as f32;
        let vector: Vec<f32> = (0..dim)
            .map(|d| centre + ((i * 7 + d * 13) % 23) as f32 * 0.02)
            .collect();
        store.push(&vector).unwrap();
    }
    store
}

fn query(dim: usize) -> Vec<f32> {
    (0..dim).map(|d| 7.0 + (d % 5) as f32 * 0.02).collect()
}

fn params(rows: usize) -> (IvfParams, PqParams) {
    (
        IvfParams::for_rows(rows),
        PqParams {
            m: 4,
            ..PqParams::default()
        },
    )
}

#[test]
fn exact_rescoring_recovers_the_true_neighbours() {
    // With every list probed and a wide rescore, the approximate stage only
    // has to *nominate* the right rows — the exact pass fixes the order. If
    // this fails, residual encoding or the distance table is wrong.
    let store = store(500, 16, Metric::L2);
    let (ivf, pq) = params(500);
    let index = IvfPqIndex::build(&store, ivf, pq)
        .unwrap()
        .with_nprobe(ivf.nlist)
        .with_rescore(64);

    let found = index.search(&store, &query(16), 5, None).unwrap();
    let truth = FlatIndex::new()
        .search(&store, &query(16), 5, None)
        .unwrap();

    let hits = found
        .iter()
        .filter(|c| truth.iter().any(|t| t.ordinal == c.ordinal))
        .count();
    assert!(hits >= 4, "recovered {hits}/5: {found:?} vs {truth:?}");
}

#[test]
fn rescored_scores_are_exact_not_approximate() {
    // A rescored candidate must carry its true distance, because the caller
    // sees the score. Returning the PQ estimate would be a number that looks
    // authoritative and is not.
    let store = store(300, 16, Metric::L2);
    let (ivf, pq) = params(300);
    let index = IvfPqIndex::build(&store, ivf, pq)
        .unwrap()
        .with_nprobe(ivf.nlist);

    let found = index.search(&store, &query(16), 3, None).unwrap();
    for candidate in &found {
        let vector = store.get(candidate.ordinal).unwrap();
        let exact = telividb_distance::score(Metric::L2, &query(16), vector);
        assert!(
            (candidate.score - exact).abs() < 1e-3,
            "score {} is not the true distance {exact}",
            candidate.score
        );
    }
}

#[test]
fn raising_nprobe_never_lowers_recall() {
    let store = store(600, 16, Metric::L2);
    let (ivf, pq) = params(600);
    let truth = FlatIndex::new()
        .search(&store, &query(16), 10, None)
        .unwrap();

    let mut previous = 0usize;
    for nprobe in [1usize, 4, ivf.nlist] {
        let index = IvfPqIndex::build(&store, ivf, pq)
            .unwrap()
            .with_nprobe(nprobe)
            .with_rescore(16);
        let found = index.search(&store, &query(16), 10, None).unwrap();
        let hits = found
            .iter()
            .filter(|c| truth.iter().any(|t| t.ordinal == c.ordinal))
            .count();
        assert!(
            hits + 1 >= previous,
            "nprobe {nprobe} recalled {hits} after {previous}"
        );
        previous = hits;
    }
}

#[test]
fn a_visibility_predicate_is_applied_during_the_scan() {
    // Invariant 15 again: filtering after selection leaks rank and count.
    let store = store(400, 16, Metric::L2);
    let (ivf, pq) = params(400);
    let index = IvfPqIndex::build(&store, ivf, pq)
        .unwrap()
        .with_nprobe(ivf.nlist);

    let allowed = |o: Ordinal| o.row().is_multiple_of(2);
    let found = index.search(&store, &query(16), 8, Some(&allowed)).unwrap();
    assert!(!found.is_empty());
    assert!(found.iter().all(|c| c.ordinal.row().is_multiple_of(2)));
}

#[test]
fn cosine_is_scored_through_the_same_table() {
    let store = store(300, 16, Metric::Cosine);
    let (ivf, pq) = params(300);
    let index = IvfPqIndex::build(&store, ivf, pq)
        .unwrap()
        .with_nprobe(ivf.nlist)
        .with_rescore(32);

    let found = index.search(&store, &query(16), 5, None).unwrap();
    assert_eq!(found.len(), 5);
    // Best-first under a higher-is-nearer metric.
    assert!(
        found.windows(2).all(|w| w[0].score >= w[1].score),
        "{found:?}"
    );
}

#[test]
fn an_empty_store_yields_an_index_that_matches_nothing() {
    let store = MemoryStore::new(Dim::new(16).unwrap(), Metric::L2);
    match IvfPqIndex::build(
        &store,
        IvfParams::default(),
        PqParams {
            m: 4,
            ..PqParams::default()
        },
    ) {
        // Either outcome is defensible; what must not happen is a codebook
        // trained on nothing, which encodes every row to the same code.
        Ok(index) => assert!(
            index
                .search(&store, &query(16), 5, None)
                .unwrap()
                .is_empty()
        ),
        Err(e) => assert!(
            e.to_string().contains("training"),
            "an empty corpus should be refused for lack of training data: {e}"
        ),
    }
}

#[test]
fn a_query_of_the_wrong_width_is_refused() {
    let store = store(300, 16, Metric::L2);
    let (ivf, pq) = params(300);
    let index = IvfPqIndex::build(&store, ivf, pq).unwrap();
    assert!(index.search(&store, &[1.0, 2.0], 5, None).is_err());
}
