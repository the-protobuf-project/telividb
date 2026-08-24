use super::*;
use telividb_core::Ordinal;

const DIM: u32 = 4;

fn dim() -> Dim {
    Dim::new(DIM).unwrap()
}

fn schema() -> Fingerprint {
    Fingerprint::of(b"schema-v1")
}

fn model() -> Fingerprint {
    Fingerprint::of(b"model-v1")
}

fn open(dir: &Path) -> VectorField {
    VectorField::open(dir, "text_bge", dim(), Metric::Dot, schema(), model()).unwrap()
}

fn vector(seed: f32) -> Vec<f32> {
    vec![seed, seed + 1.0, seed + 2.0, seed + 3.0]
}

/// Read every row out of a field, in row order, by walking its stores.
fn all_rows(field: &VectorField) -> Vec<Vec<f32>> {
    let mut out = Vec::new();
    for store in field.stores() {
        for row in 0..store.len() {
            if let Some(v) = store.get(Ordinal::from_row(row as u32)) {
                out.push(v.to_vec());
            }
        }
    }
    out
}

#[test]
fn appended_vectors_are_immediately_searchable() {
    // The buffer is searchable by design: waiting for a seal would make "I
    // just wrote it and cannot find it" the normal experience.
    let dir = tempfile::tempdir().unwrap();
    let mut field = open(dir.path());
    field.append(&vector(0.0)).unwrap();
    field.append(&vector(10.0)).unwrap();

    assert_eq!(field.rows(), 2);
    assert_eq!(all_rows(&field), vec![vector(0.0), vector(10.0)]);
}

#[test]
fn rows_survive_a_reopen_through_the_wal() {
    // The whole point of writing the log before the buffer: an acknowledged
    // write must outlive the process even with nothing sealed.
    let dir = tempfile::tempdir().unwrap();
    {
        let mut field = open(dir.path());
        field.append(&vector(1.0)).unwrap();
        field.append(&vector(2.0)).unwrap();
        field.commit().unwrap();
    }

    let reopened = open(dir.path());
    assert_eq!(reopened.rows(), 2, "wal replay lost rows");
    assert_eq!(all_rows(&reopened), vec![vector(1.0), vector(2.0)]);
}

#[test]
fn sealing_moves_rows_into_a_segment_without_losing_any() {
    let dir = tempfile::tempdir().unwrap();
    let mut field = open(dir.path());
    for i in 0..5 {
        field.append(&vector(i as f32 * 10.0)).unwrap();
    }
    field.commit().unwrap();
    field.seal().unwrap();

    assert_eq!(field.rows(), 5, "sealing must not change the row count");
    assert_eq!(
        field.buffered_bytes(),
        0,
        "the buffer is empty after a seal"
    );
    assert_eq!(all_rows(&field).len(), 5);
}

#[test]
fn sealed_rows_survive_a_reopen_through_the_manifest() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut field = open(dir.path());
        field.append(&vector(7.0)).unwrap();
        field.commit().unwrap();
        field.seal().unwrap();
    }

    let reopened = open(dir.path());
    assert_eq!(reopened.rows(), 1);
    assert_eq!(all_rows(&reopened), vec![vector(7.0)]);
}

#[test]
fn a_seal_does_not_leave_the_log_to_be_replayed_twice() {
    // Without truncating the log at seal time, reopening would re-add rows the
    // segment already holds — silently doubling the corpus.
    let dir = tempfile::tempdir().unwrap();
    {
        let mut field = open(dir.path());
        field.append(&vector(3.0)).unwrap();
        field.commit().unwrap();
        field.seal().unwrap();
    }

    let reopened = open(dir.path());
    assert_eq!(
        reopened.rows(),
        1,
        "the sealed row was replayed a second time"
    );
}

#[test]
fn sealed_and_buffered_rows_coexist_after_a_seal() {
    let dir = tempfile::tempdir().unwrap();
    let mut field = open(dir.path());
    field.append(&vector(1.0)).unwrap();
    field.commit().unwrap();
    field.seal().unwrap();
    field.append(&vector(2.0)).unwrap();
    field.commit().unwrap();

    assert_eq!(field.rows(), 2);
    assert_eq!(all_rows(&field), vec![vector(1.0), vector(2.0)]);

    let reopened = open(dir.path());
    assert_eq!(reopened.rows(), 2, "one sealed plus one replayed");
}

#[test]
fn append_returns_a_row_that_spans_sealed_and_buffered() {
    // Segment ordinals are segment-local (invariant 9), so the row a caller
    // records has to be offset past everything already sealed.
    let dir = tempfile::tempdir().unwrap();
    let mut field = open(dir.path());
    assert_eq!(field.append(&vector(1.0)).unwrap(), 0);
    field.commit().unwrap();
    field.seal().unwrap();
    assert_eq!(
        field.append(&vector(2.0)).unwrap(),
        1,
        "the row after a sealed one continues the numbering"
    );
}

#[test]
fn row_of_maps_a_store_hit_back_to_a_field_wide_row() {
    let dir = tempfile::tempdir().unwrap();
    let mut field = open(dir.path());
    field.append(&vector(1.0)).unwrap();
    field.commit().unwrap();
    field.seal().unwrap();
    field.append(&vector(2.0)).unwrap();

    // Store 0 is the sealed segment, store 1 the buffer.
    assert_eq!(field.row_of(0, Ordinal::from_row(0)), 0);
    assert_eq!(field.row_of(1, Ordinal::from_row(0)), 1);
}

#[test]
fn seal_if_needed_respects_the_threshold() {
    let dir = tempfile::tempdir().unwrap();
    let mut field = open(dir.path());
    field.append(&vector(1.0)).unwrap();

    assert!(
        !field.seal_if_needed(usize::MAX).unwrap(),
        "a buffer under the threshold must not seal"
    );
    assert!(
        field.seal_if_needed(0).unwrap(),
        "a buffer over the threshold must seal"
    );
    assert_eq!(field.buffered_bytes(), 0);
}

#[test]
fn an_empty_buffer_never_seals() {
    // Sealing nothing would write an empty segment and grow the manifest for
    // no reason.
    let dir = tempfile::tempdir().unwrap();
    let mut field = open(dir.path());
    assert!(!field.seal_if_needed(0).unwrap());
}

#[test]
fn a_fresh_field_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(open(dir.path()).rows(), 0);
}
