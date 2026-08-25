use super::*;
use crate::adapters::MemoryStore;
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
fn every_present_row_is_encoded_into_exactly_one_list() {
    let store = store(600, 16, Metric::L2);
    let (ivf, pq) = params(600);
    let index = IvfPqIndex::build(&store, ivf, pq).unwrap();

    let rows: usize = index.lists.iter().map(|l| l.rows.len()).sum();
    let codes: usize = index.lists.iter().map(|l| l.codes.len()).sum();
    assert_eq!(rows, 600);
    assert_eq!(codes, 600 * index.bytes_per_row(), "one code run per row");
}

#[test]
fn a_row_costs_m_bytes_rather_than_a_full_vector() {
    // The point of PQ. 16 dims of f32 is 64 bytes; four subspaces is four.
    let store = store(300, 16, Metric::L2);
    let (ivf, pq) = params(300);
    let index = IvfPqIndex::build(&store, ivf, pq).unwrap();

    assert_eq!(index.bytes_per_row(), 4);
    assert!(
        index.bytes_per_row() * 16 <= 16 * 4,
        "should beat f32 by 16x"
    );
}
