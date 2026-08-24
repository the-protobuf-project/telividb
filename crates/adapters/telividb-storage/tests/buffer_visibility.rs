//! A write must be findable before it is sealed.
//!
//! This is the behaviour the searchable buffer exists for. Without it a row is
//! invisible until enough rows accumulate to trip the seal threshold — which is
//! tolerable during bulk import and indefensible for interactive ingest, where
//! "I wrote it and cannot find it" is the first thing anyone hits.

use telividb_core::{Dim, Metric, VectorStore};
use telividb_index::{FlatIndex, Source, VectorIndex, adapters::MemoryStore, merge_top_k};
use telividb_storage::MutableBuffer;

const DIM: usize = 4;

fn dim() -> Dim {
    Dim::new(DIM as u32).unwrap()
}

/// A sealed segment standing in for one on disk.
fn sealed_segment(rows: &[[f32; DIM]]) -> MemoryStore {
    let mut store = MemoryStore::new(dim(), Metric::Dot);
    for row in rows {
        store.push(row).unwrap();
    }
    store
}

/// Search buffer and segment, then merge — the shape every real query takes.
fn search_both(
    buffer: &MutableBuffer,
    segment: &MemoryStore,
    query: &[f32],
    k: usize,
) -> telividb_index::Merged {
    let from_buffer = FlatIndex::new().search(buffer, query, k, None).unwrap();
    let from_sealed = FlatIndex::new().search(segment, query, k, None).unwrap();
    merge_top_k(
        &[
            (Source::Buffer, from_buffer),
            (Source::Sealed(1), from_sealed),
        ],
        k,
        Metric::Dot.higher_is_nearer(),
    )
}

#[test]
fn an_unsealed_write_is_immediately_findable() {
    let segment = sealed_segment(&[[0.1, 0.0, 0.0, 0.0]]);
    let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
    buffer.push(&[1.0, 0.0, 0.0, 0.0]).unwrap();

    let merged = search_both(&buffer, &segment, &[1.0, 0.0, 0.0, 0.0], 5);

    assert_eq!(merged.hits[0].source, Source::Buffer);
    assert_eq!(merged.stats.buffer_hits, 1);
    assert_eq!(merged.stats.sealed_hits, 1);
}

#[test]
fn a_better_sealed_match_still_outranks_the_buffer() {
    // Recency must not be mistaken for relevance.
    let segment = sealed_segment(&[[1.0, 0.0, 0.0, 0.0]]);
    let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
    buffer.push(&[0.2, 0.0, 0.0, 0.0]).unwrap();

    let merged = search_both(&buffer, &segment, &[1.0, 0.0, 0.0, 0.0], 5);

    assert_eq!(merged.hits[0].source, Source::Sealed(1));
    assert_eq!(merged.hits[1].source, Source::Buffer);
}

#[test]
fn top_k_is_taken_across_both_sources() {
    // Four strong buffer rows and one weak sealed row: k=3 must be all buffer,
    // not "k from each" or a fixed split between sources.
    let segment = sealed_segment(&[[0.01, 0.0, 0.0, 0.0]]);
    let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
    for scale in [0.9f32, 0.8, 0.7, 0.6] {
        buffer.push(&[scale, 0.0, 0.0, 0.0]).unwrap();
    }

    let merged = search_both(&buffer, &segment, &[1.0, 0.0, 0.0, 0.0], 3);

    assert_eq!(merged.hits.len(), 3);
    assert_eq!(merged.stats.buffer_hits, 3);
    assert_eq!(merged.stats.sealed_hits, 0);
}

#[test]
fn an_empty_buffer_changes_nothing() {
    let segment = sealed_segment(&[[1.0, 0.0, 0.0, 0.0], [0.5, 0.0, 0.0, 0.0]]);
    let buffer = MutableBuffer::new(dim(), Metric::Dot);

    let merged = search_both(&buffer, &segment, &[1.0, 0.0, 0.0, 0.0], 5);

    assert_eq!(merged.hits.len(), 2);
    assert_eq!(merged.stats.buffer_hits, 0);
    assert!(!merged.stats.is_fully_exact());
}

#[test]
fn results_are_attributed_so_recall_stays_measurable() {
    // Buffer hits are exhaustive. Counted as index hits they would inflate any
    // recall measurement and hide a genuinely degraded index.
    let segment = sealed_segment(&[[0.5, 0.0, 0.0, 0.0]]);
    let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
    buffer.push(&[0.9, 0.0, 0.0, 0.0]).unwrap();

    let merged = search_both(&buffer, &segment, &[1.0, 0.0, 0.0, 0.0], 5);

    assert!(!merged.stats.is_fully_exact(), "a sealed hit is present");
    assert_eq!(
        merged.stats.buffer_hits + merged.stats.sealed_hits,
        merged.hits.len(),
        "every returned hit must be attributed"
    );
}

#[test]
fn absent_rows_in_the_buffer_are_not_returned() {
    let segment = sealed_segment(&[[0.1, 0.0, 0.0, 0.0]]);
    let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
    buffer.push_absent();
    buffer.push(&[0.9, 0.0, 0.0, 0.0]).unwrap();

    let merged = search_both(&buffer, &segment, &[1.0, 0.0, 0.0, 0.0], 5);

    assert_eq!(merged.stats.buffer_hits, 1, "the absent row must not score");
    assert_eq!(buffer.len(), 2, "but it still occupies an ordinal");
}

#[test]
fn clearing_after_seal_removes_the_rows_from_the_buffer() {
    let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
    buffer.push(&[1.0, 0.0, 0.0, 0.0]).unwrap();
    assert_eq!(buffer.len(), 1);

    // Only ever after the segment is durable and the manifest names it —
    // otherwise the rows disappear from both places at once.
    buffer.clear();

    let hits = FlatIndex::new()
        .search(&buffer, &[1.0, 0.0, 0.0, 0.0], 5, None)
        .unwrap();
    assert!(hits.is_empty());
}
