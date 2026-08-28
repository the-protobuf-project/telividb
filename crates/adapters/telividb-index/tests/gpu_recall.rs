//! The GPU index must find neighbours as good as exhaustive CPU search.
//!
//! Both indexes score every row, so this asserts **score agreement, not a
//! recall threshold** — unlike HNSW, where agreement is approximate by
//! construction. That is stricter than invariant 8 requires: any divergence
//! beyond floating-point noise is a bug in the tensor path, not a tuning
//! parameter. See `assert_same_quality` for why the comparison is on scores
//! rather than ordinals.
//!
//! Runs on whatever device `Device::best` selects, so on a Mac built with the
//! `metal` feature this is genuinely exercising Metal, and on CI it exercises
//! the CPU fallback. Both must produce the same answers as `FlatIndex`.

#![cfg(feature = "gpu")]

mod support;

use support::{DIM, corpus};
use telividb_core::{Metric, Ordinal, VectorStore};
use telividb_index::adapters::{Device, FlatIndex, GpuFlatIndex};
use telividb_index::{VectorIndex, adapters::MemoryStore};

const ROWS: usize = 2_000;
const K: usize = 10;

fn both(store: &MemoryStore) -> (FlatIndex, GpuFlatIndex) {
    (FlatIndex::new(), GpuFlatIndex::build(store).unwrap())
}

/// One ULP at the magnitudes this fixture produces (scores near 10, where f32
/// resolution is ~1e-6). Generous enough to absorb summation-order differences,
/// far tighter than the gap between a correct neighbour and a wrong one.
const TOLERANCE: f32 = 1e-4;

/// Assert the GPU found neighbours **as good as** the CPU's, rank by rank.
///
/// Deliberately not an ordinal-by-ordinal comparison. Exact ties are common —
/// clustered fixtures produce genuinely equidistant rows — and the CPU's
/// scalar loop and the device's fused matmul break them differently: measured
/// here, two rows the CPU scored *identically* differed by 9.5e-7 on the GPU,
/// flipping their order. Neither ranking is wrong, so asserting on ordinals
/// would be testing floating-point summation order rather than the index.
///
/// Comparing scores per rank keeps the real property — equally good results —
/// while staying insensitive to how ties fall.
fn assert_same_quality(
    expected: &[telividb_index::Candidate],
    actual: &[telividb_index::Candidate],
) {
    assert_eq!(actual.len(), expected.len(), "same number of hits");
    for (rank, (want, got)) in expected.iter().zip(actual.iter()).enumerate() {
        assert!(
            (want.score - got.score).abs() <= TOLERANCE,
            "rank {rank}: cpu scored {} ({}), gpu scored {} ({})",
            want.score,
            want.ordinal.row(),
            got.score,
            got.ordinal.row(),
        );
    }
}

#[test]
fn gpu_returns_the_same_neighbours_as_exhaustive_cpu_search() {
    let (store, queries) = corpus(ROWS, Metric::Dot, 0xA11CE);
    let (flat, gpu) = both(&store);

    eprintln!("gpu index device: {}", gpu.device());

    let mut compared = 0;
    for query in &queries {
        let expected = flat.search(&store, query, K, None).unwrap();
        let actual = gpu.search(&store, query, K, None).unwrap();

        assert_same_quality(&expected, &actual);
        compared += 1;
    }
    assert!(compared > 0, "the fixture produced no queries");
}

#[test]
fn agreement_holds_under_a_visibility_filter() {
    // The filtered path is where a GPU index is most likely to diverge: it
    // scores everything and excludes afterwards, where the CPU index skips
    // rows during the scan. Both must still land on the same k.
    let (store, queries) = corpus(ROWS, Metric::Dot, 0xF117E);
    let (flat, gpu) = both(&store);

    // Keep two thirds — selective enough to matter, permissive enough that a
    // full k is still available.
    let visible = |o: Ordinal| !o.row().is_multiple_of(3);

    for query in queries.iter().take(10) {
        let expected = flat.search(&store, query, K, Some(&visible)).unwrap();
        let actual = gpu.search(&store, query, K, Some(&visible)).unwrap();

        assert_same_quality(&expected, &actual);
        assert!(
            actual.iter().all(|c| !c.ordinal.row().is_multiple_of(3)),
            "a hidden row reached the results"
        );
        assert_eq!(actual.len(), K, "filtering must not shrink k");
    }
}

#[test]
fn agreement_holds_when_rows_are_absent() {
    // A multimodal collection where only some points carry this field.
    let mut store = MemoryStore::new(telividb_core::Dim::new(DIM as u32).unwrap(), Metric::Dot);
    let mut rng = support::Rng(0xAB5E17);
    for row in 0..500u32 {
        if row.is_multiple_of(4) {
            store.push_absent();
        } else {
            store.push(&rng.vector()).unwrap();
        }
    }

    let (flat, gpu) = both(&store);
    for _ in 0..10 {
        let query = rng.vector();
        let expected = flat.search(&store, &query, K, None).unwrap();
        let actual = gpu.search(&store, &query, K, None).unwrap();

        assert_same_quality(&expected, &actual);
        assert!(
            actual.iter().all(|c| store.get(c.ordinal).is_some()),
            "an absent row was returned"
        );
    }
}

#[test]
fn cosine_agrees_too() {
    // Cosine normalises at ingest and is scored as dot, so this checks the
    // normalisation survives the GGUF round trip rather than a second kernel.
    let (store, queries) = corpus(ROWS, Metric::Cosine, 0xC05);
    let (flat, gpu) = both(&store);

    for query in queries.iter().take(10) {
        let expected = flat.search(&store, query, K, None).unwrap();
        let actual = gpu.search(&store, query, K, None).unwrap();
        assert_same_quality(&expected, &actual);
    }
}

#[test]
fn the_selected_device_is_reported() {
    // Not a correctness property — an observability one. A GPU index that
    // silently fell back to CPU passes every test above while delivering none
    // of the speed, so the device it chose has to be inspectable.
    let name = Device::best().kind().as_str();
    assert!(matches!(name, "metal" | "cuda" | "cpu"));
}
