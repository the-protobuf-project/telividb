use super::*;
use telividb_core::ResourceName;

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

#[test]
fn a_bare_point_round_trips() {
    let point = Point::new(name("collections/media/points/doc-1"));
    let bytes = encode(&point);
    let decoded = decode(name("collections/media/points/doc-1"), &bytes).unwrap();
    assert_eq!(decoded, point);
}

#[test]
fn a_span_round_trips() {
    let point = Point::new(name("a/1")).with_span(Span::new(100, 200).unwrap());
    let bytes = encode(&point);
    let decoded = decode(name("a/1"), &bytes).unwrap();
    assert_eq!(decoded.span, Some(Span::new(100, 200).unwrap()));
}

#[test]
fn a_full_content_ref_round_trips() {
    let mut content_ref = ContentRef::uri("s3://bucket/key").with_inline("hello world");
    content_ref.byte_range = Some((10, 20));
    content_ref.sha256 = Some([7u8; 32]);
    let point = Point::new(name("a/1")).with_content_ref(content_ref.clone());

    let bytes = encode(&point);
    let decoded = decode(name("a/1"), &bytes).unwrap();
    assert_eq!(decoded.content_ref, Some(content_ref));
}

#[test]
fn a_minimal_content_ref_round_trips() {
    let content_ref = ContentRef::uri("file:///tmp/x");
    let point = Point::new(name("a/1")).with_content_ref(content_ref.clone());

    let bytes = encode(&point);
    let decoded = decode(name("a/1"), &bytes).unwrap();
    assert_eq!(decoded.content_ref, Some(content_ref));
}

#[test]
fn truncated_bytes_are_reported_not_panicked_on() {
    let point = Point::new(name("a/1")).with_span(Span::new(0, 100).unwrap());
    let mut bytes = encode(&point);
    bytes.truncate(bytes.len() - 1);
    assert!(decode(name("a/1"), &bytes).is_err());
}
