use super::*;
use crate::domain::{ContentRef, ResourceName, Span};

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

#[test]
fn a_new_point_has_neither_span_nor_content_ref() {
    let point = Point::new(name("collections/media/points/doc-1"));
    assert_eq!(point.name, name("collections/media/points/doc-1"));
    assert!(point.span.is_none());
    assert!(point.content_ref.is_none());
}

#[test]
fn builder_methods_attach_span_and_content_ref() {
    let span = Span::new(0, 100).unwrap();
    let content_ref = ContentRef::uri("s3://bucket/key");
    let point = Point::new(name("a/1"))
        .with_span(span)
        .with_content_ref(content_ref.clone());

    assert_eq!(point.span, Some(span));
    assert_eq!(point.content_ref, Some(content_ref));
}
