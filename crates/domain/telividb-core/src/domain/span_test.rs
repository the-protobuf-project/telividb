use super::*;

fn span(a: u64, b: u64) -> Span {
    Span::new(a, b).unwrap()
}

#[test]
fn inverted_span_is_rejected() {
    assert!(Span::new(900, 100).is_err());
}

#[test]
fn empty_span_is_allowed() {
    assert_eq!(span(100, 100).duration_ms(), 0);
}

#[test]
fn touching_spans_do_not_overlap() {
    assert!(!span(0, 100).overlaps(span(100, 200)));
}

#[test]
fn overlap_is_symmetric() {
    assert!(span(0, 150).overlaps(span(100, 200)));
    assert!(span(100, 200).overlaps(span(0, 150)));
}

#[test]
fn containment_is_not_overlap_alone() {
    assert!(span(0, 500).contains(span(100, 200)));
    assert!(!span(100, 200).contains(span(0, 500)));
}
