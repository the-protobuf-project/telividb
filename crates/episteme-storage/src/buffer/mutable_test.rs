use super::*;

fn buf(metric: Metric) -> MutableBuffer {
    MutableBuffer::new(Dim::new(3).unwrap(), metric)
}

#[test]
fn starts_empty() {
    let b = buf(Metric::Dot);
    assert_eq!(b.rows(), 0);
    assert!(b.is_empty());
}

#[test]
fn push_returns_sequential_ordinals() {
    let mut b = buf(Metric::Dot);
    assert_eq!(b.push(&[1.0, 0.0, 0.0]).unwrap().row(), 0);
    assert_eq!(b.push(&[0.0, 1.0, 0.0]).unwrap().row(), 1);
    assert_eq!(b.rows(), 2);
}

#[test]
fn is_searchable_immediately_after_a_write() {
    // The property the whole type exists for: a row is visible the instant it
    // is accepted, not once a seal threshold trips.
    let mut b = buf(Metric::Dot);
    let o = b.push(&[0.5, 0.5, 0.5]).unwrap();
    assert_eq!(b.get(o), Some([0.5, 0.5, 0.5].as_slice()));
}

#[test]
fn dimension_mismatch_is_rejected() {
    let mut b = buf(Metric::Dot);
    assert!(matches!(
        b.push(&[1.0, 0.0]),
        Err(Error::DimMismatch {
            expected: 3,
            actual: 2
        })
    ));
    assert_eq!(b.rows(), 0, "a rejected write must not be recorded");
}

#[test]
fn non_finite_components_are_rejected() {
    let mut b = buf(Metric::Dot);
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(
            matches!(b.push(&[1.0, bad, 0.0]), Err(Error::NonFinite { index: 1 })),
            "{bad} was accepted"
        );
    }
    assert_eq!(b.rows(), 0);
}

#[test]
fn cosine_normalises_on_ingest() {
    let mut b = buf(Metric::Cosine);
    let o = b.push(&[3.0, 4.0, 0.0]).unwrap();
    let stored = b.get(o).unwrap();
    let norm: f32 = stored.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-6, "not unit length: {norm}");
}

#[test]
fn dot_does_not_normalise() {
    let mut b = buf(Metric::Dot);
    let o = b.push(&[3.0, 4.0, 0.0]).unwrap();
    assert_eq!(b.get(o), Some([3.0, 4.0, 0.0].as_slice()));
}

#[test]
fn absent_rows_occupy_an_ordinal_but_return_none() {
    let mut b = buf(Metric::Dot);
    b.push(&[1.0, 0.0, 0.0]).unwrap();
    let absent = b.push_absent();
    b.push(&[0.0, 0.0, 1.0]).unwrap();

    assert_eq!(b.rows(), 3, "the absent row still holds its position");
    assert_eq!(b.get(absent), None, "must not be scored as a zero vector");
    assert!(b.get(Ordinal::from_row(2)).is_some());
}

#[test]
fn seal_threshold_tracks_bytes_not_rows() {
    let mut b = buf(Metric::Dot);
    assert!(!b.should_seal(1024));
    for _ in 0..100 {
        b.push(&[1.0, 2.0, 3.0]).unwrap();
    }
    assert!(b.bytes() >= 100 * 3 * 4);
    assert!(b.should_seal(1024));
}

#[test]
fn clear_empties_the_buffer_and_resets_ordinals() {
    let mut b = buf(Metric::Dot);
    b.push(&[1.0, 0.0, 0.0]).unwrap();
    b.push(&[0.0, 1.0, 0.0]).unwrap();
    b.clear();

    assert_eq!(b.rows(), 0);
    assert_eq!(b.get(Ordinal::from_row(0)), None);
    assert_eq!(
        b.push(&[0.0, 0.0, 1.0]).unwrap().row(),
        0,
        "numbering restarts"
    );
}

#[test]
fn with_capacity_does_not_pre_create_rows() {
    let b = MutableBuffer::with_capacity(Dim::new(768).unwrap(), Metric::Cosine, 10_000);
    assert_eq!(b.rows(), 0, "capacity is not length");
}

#[test]
fn reading_past_the_end_returns_none() {
    let b = buf(Metric::Dot);
    assert_eq!(b.get(Ordinal::from_row(99)), None);
}
