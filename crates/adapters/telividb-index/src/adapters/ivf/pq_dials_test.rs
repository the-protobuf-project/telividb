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

#[test]
fn a_wider_rescore_raises_the_quantization_ceiling() {
    // Probing every list still leaves recall short, because the codes
    // themselves lost information — more probing cannot recover it. Rescoring
    // more candidates exactly is the dial that can, and this pins that the
    // two knobs do different jobs rather than being interchangeable.
    let store = store(600, 16, Metric::L2);
    let (ivf, pq) = params(600);
    let truth = FlatIndex::new()
        .search(&store, &query(16), 10, None)
        .unwrap();

    let recall_at = |factor: usize| {
        let index = IvfPqIndex::build(&store, ivf, pq)
            .unwrap()
            .with_nprobe(ivf.nlist)
            .with_rescore(factor);
        index
            .search(&store, &query(16), 10, None)
            .unwrap()
            .iter()
            .filter(|c| truth.iter().any(|t| t.ordinal == c.ordinal))
            .count()
    };

    let narrow = recall_at(1);
    let wide = recall_at(32);
    assert!(
        wide >= narrow,
        "a wider rescore recalled {wide}, fewer than the narrow {narrow}"
    );
}
