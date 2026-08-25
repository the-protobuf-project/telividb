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
