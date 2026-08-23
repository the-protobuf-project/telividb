//! Sealing a buffer and reading it back.
//!
//! This closes the Phase 1 loop: rows accepted into the unsealed buffer, sealed
//! into an immutable segment, reopened from disk, and searched — with the index
//! unable to tell which side it was handed.

use episteme_core::{Dim, Fingerprint, Metric, Ordinal, VectorStore};
use episteme_index::{FlatIndex, VectorIndex};
use episteme_storage::{MutableBuffer, SegmentReader, SegmentWriter};

const DIM: u32 = 4;

fn dim() -> Dim {
    Dim::new(DIM).unwrap()
}

fn schema() -> Fingerprint {
    Fingerprint::of(b"descriptor-set-v1")
}

fn model() -> Fingerprint {
    Fingerprint::of(b"bge-large.gguf")
}

/// Fill a buffer, seal it, and reopen the field.
fn seal_and_reopen(rows: &[Option<[f32; 4]>], dir: &std::path::Path) -> SegmentReader {
    let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
    for row in rows {
        match row {
            Some(v) => {
                buffer.push(v).unwrap();
            }
            None => {
                buffer.push_absent();
            }
        }
    }

    let path = dir.join("seg_00001");
    let mut writer = SegmentWriter::create(&path, schema()).unwrap();
    writer.write_field("text_bge", &buffer, model()).unwrap();
    let sealed = writer.finish().unwrap();

    SegmentReader::open_field(&sealed, "text_bge", schema()).unwrap()
}

#[test]
fn vectors_survive_the_round_trip_exactly() {
    let dir = tempfile::tempdir().unwrap();
    let reader = seal_and_reopen(
        &[
            Some([1.0, 0.0, 0.0, 0.0]),
            Some([0.5, 0.25, 0.125, 0.0625]),
            Some([-1.0, 2.5, -3.75, 4.0]),
        ],
        dir.path(),
    );

    assert_eq!(reader.len(), 3);
    assert_eq!(
        reader.get(Ordinal::from_row(0)).unwrap(),
        &[1.0, 0.0, 0.0, 0.0]
    );
    assert_eq!(
        reader.get(Ordinal::from_row(1)).unwrap(),
        &[0.5, 0.25, 0.125, 0.0625]
    );
    assert_eq!(
        reader.get(Ordinal::from_row(2)).unwrap(),
        &[-1.0, 2.5, -3.75, 4.0],
        "sign and magnitude must survive byte-exactly"
    );
}

#[test]
fn field_metadata_survives() {
    let dir = tempfile::tempdir().unwrap();
    let reader = seal_and_reopen(&[Some([1.0, 0.0, 0.0, 0.0])], dir.path());

    assert_eq!(reader.dim(), dim());
    assert_eq!(reader.metric(), Metric::Dot);
    assert_eq!(reader.header().model_fingerprint, model());
}

#[test]
fn absent_rows_come_back_absent() {
    // The bytes still occupy their slot so fixed stride holds, but the row must
    // not be scored — a zero vector would rank like any other.
    let dir = tempfile::tempdir().unwrap();
    let reader = seal_and_reopen(
        &[Some([1.0, 0.0, 0.0, 0.0]), None, Some([0.0, 0.0, 0.0, 1.0])],
        dir.path(),
    );

    assert_eq!(reader.len(), 3, "the absent row keeps its ordinal");
    assert!(reader.get(Ordinal::from_row(1)).is_none());
    assert!(reader.get(Ordinal::from_row(2)).is_some());
}

#[test]
fn rows_start_on_a_64_byte_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let reader = seal_and_reopen(&[Some([1.0, 0.0, 0.0, 0.0])], dir.path());
    assert_eq!(reader.layout().data_offset % 64, 0);
}

#[test]
fn a_sealed_segment_is_searchable_through_the_same_port() {
    let dir = tempfile::tempdir().unwrap();
    let reader = seal_and_reopen(
        &[
            Some([0.1, 0.0, 0.0, 0.0]),
            Some([1.0, 0.0, 0.0, 0.0]),
            Some([0.5, 0.0, 0.0, 0.0]),
        ],
        dir.path(),
    );

    let hits = FlatIndex
        .search(&reader, &[1.0, 0.0, 0.0, 0.0], 2, None)
        .unwrap();
    assert_eq!(hits[0].ordinal.row(), 1);
    assert_eq!(hits[1].ordinal.row(), 2);
}

#[test]
fn a_drifted_schema_is_refused_before_any_bytes_are_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("seg_00001");
    let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
    buffer.push(&[1.0, 0.0, 0.0, 0.0]).unwrap();

    let mut writer = SegmentWriter::create(&path, schema()).unwrap();
    writer.write_field("text_bge", &buffer, model()).unwrap();
    let sealed = writer.finish().unwrap();

    let err = SegmentReader::open_field(&sealed, "text_bge", Fingerprint::of(b"v2")).unwrap_err();
    assert!(matches!(err, episteme_storage::Error::SchemaDrift { .. }));
}

#[test]
fn a_drifted_model_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let reader = seal_and_reopen(&[Some([1.0, 0.0, 0.0, 0.0])], dir.path());
    let err = reader
        .check_model(Fingerprint::of(b"e5-large.gguf"))
        .unwrap_err();
    assert!(matches!(err, episteme_storage::Error::ModelDrift { .. }));
}

#[test]
fn a_partial_seal_leaves_no_segment_behind() {
    // Crash mid-seal: the writer is dropped without finish(). Only a temp
    // directory may remain, never something the manifest could name.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("seg_00001");
    {
        let mut writer = SegmentWriter::create(&path, schema()).unwrap();
        let mut buffer = MutableBuffer::new(dim(), Metric::Dot);
        buffer.push(&[1.0, 0.0, 0.0, 0.0]).unwrap();
        writer.write_field("text_bge", &buffer, model()).unwrap();
        // no finish()
    }
    assert!(!path.exists(), "an unfinished segment must not appear");
}

#[test]
fn an_empty_field_seals_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let reader = seal_and_reopen(&[], dir.path());
    assert_eq!(reader.len(), 0);
    assert!(reader.is_empty());
    let hits = FlatIndex
        .search(&reader, &[1.0, 0.0, 0.0, 0.0], 5, None)
        .unwrap();
    assert!(hits.is_empty());
}
