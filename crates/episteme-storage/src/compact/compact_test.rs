use super::*;
use crate::buffer::MutableBuffer;
use episteme_core::{Dim, Metric};

fn store(rows: &[&[f32]]) -> MutableBuffer {
    let mut b = MutableBuffer::new(Dim::new(2).unwrap(), Metric::Dot);
    for r in rows {
        b.push(r).unwrap();
    }
    b
}

fn all_live(_: usize, _: Ordinal) -> bool {
    true
}

#[test]
fn a_clean_segment_survives_intact() {
    let a = store(&[&[1.0, 0.0], &[0.0, 1.0]]);
    let inputs: Vec<&dyn VectorStore> = vec![&a];
    let (out, result) =
        compact_field(&inputs, &all_live, Fingerprint::unset(), Codec::None).unwrap();

    assert_eq!(out.len(), 2);
    assert_eq!(result.rows_written, 2);
    assert_eq!(result.rows_reclaimed, 0);
}

#[test]
fn tombstoned_rows_are_dropped() {
    let a = store(&[&[1.0, 0.0], &[0.0, 1.0], &[0.5, 0.5]]);
    let inputs: Vec<&dyn VectorStore> = vec![&a];
    let drop_middle = |_: usize, o: Ordinal| o.row() != 1;

    let (out, result) =
        compact_field(&inputs, &drop_middle, Fingerprint::unset(), Codec::None).unwrap();

    assert_eq!(out.len(), 2);
    assert_eq!(result.rows_read, 3);
    assert_eq!(result.rows_reclaimed, 1);
}

#[test]
fn ordinals_are_renumbered() {
    // Positions move. Anything that stored an ordinal externally would now
    // point at a different row — which is why they must never escape.
    let a = store(&[&[9.0, 9.0], &[1.0, 0.0]]);
    let inputs: Vec<&dyn VectorStore> = vec![&a];
    let drop_first = |_: usize, o: Ordinal| o.row() != 0;

    let (out, _) = compact_field(&inputs, &drop_first, Fingerprint::unset(), Codec::None).unwrap();
    assert_eq!(out.get(Ordinal::from_row(0)), Some([1.0, 0.0].as_slice()));
}

#[test]
fn several_segments_merge_in_order() {
    let a = store(&[&[1.0, 0.0]]);
    let b = store(&[&[0.0, 1.0]]);
    let inputs: Vec<&dyn VectorStore> = vec![&a, &b];

    let (out, result) =
        compact_field(&inputs, &all_live, Fingerprint::unset(), Codec::None).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(result.rows_read, 2);
    assert_eq!(out.get(Ordinal::from_row(0)), Some([1.0, 0.0].as_slice()));
    assert_eq!(out.get(Ordinal::from_row(1)), Some([0.0, 1.0].as_slice()));
}

#[test]
fn the_live_predicate_sees_which_input_a_row_came_from() {
    // Tombstones are per segment, so the predicate must be able to tell two
    // segments' row zero apart.
    let a = store(&[&[1.0, 0.0]]);
    let b = store(&[&[0.0, 1.0]]);
    let inputs: Vec<&dyn VectorStore> = vec![&a, &b];
    let drop_first_input = |i: usize, _: Ordinal| i != 0;

    let (out, _) =
        compact_field(&inputs, &drop_first_input, Fingerprint::unset(), Codec::None).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out.get(Ordinal::from_row(0)), Some([0.0, 1.0].as_slice()));
}

#[test]
fn absent_rows_survive_as_absent() {
    // They keep their slot so ordinals stay aligned across the fields of the
    // segment being written.
    let mut a = MutableBuffer::new(Dim::new(2).unwrap(), Metric::Dot);
    a.push(&[1.0, 0.0]).unwrap();
    a.push_absent();
    let inputs: Vec<&dyn VectorStore> = vec![&a];

    let (out, result) =
        compact_field(&inputs, &all_live, Fingerprint::unset(), Codec::None).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out.get(Ordinal::from_row(1)), None);
    assert_eq!(result.rows_reclaimed, 0, "absent is not deleted");
}

#[test]
fn no_inputs_is_not_an_error() {
    let (out, result) = compact_field(&[], &all_live, Fingerprint::unset(), Codec::None).unwrap();
    assert!(out.is_empty());
    assert_eq!(result.rows_read, 0);
    assert_eq!(result.reclaimed_fraction(), 0.0);
}

#[test]
fn reclaimed_fraction_reports_whether_the_run_was_worth_it() {
    let a = store(&[&[1.0, 0.0], &[0.0, 1.0], &[0.5, 0.5], &[0.1, 0.1]]);
    let inputs: Vec<&dyn VectorStore> = vec![&a];
    let keep_half = |_: usize, o: Ordinal| o.row() < 2;

    let (_, result) = compact_field(&inputs, &keep_half, Fingerprint::unset(), Codec::None).unwrap();
    assert_eq!(result.reclaimed_fraction(), 0.5);
}
