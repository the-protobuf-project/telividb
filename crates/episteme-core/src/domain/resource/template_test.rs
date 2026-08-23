use super::*;

fn point_template() -> Template {
    Template::compile("collections/{collection}/points/{point}").unwrap()
}

#[test]
fn round_trips_through_format_and_parse() {
    let t = point_template();
    let name = t.format(&["finance", "doc-123"]).unwrap();
    assert_eq!(name.as_str(), "collections/finance/points/doc-123");

    let bound = t.parse(&name).unwrap();
    assert_eq!(bound, vec![("collection", "finance"), ("point", "doc-123")]);
}

#[test]
fn lists_placeholders_in_order() {
    let t = point_template();
    assert_eq!(
        t.placeholders().collect::<Vec<_>>(),
        vec!["collection", "point"]
    );
}

#[test]
fn parse_rejects_a_different_shape() {
    let t = point_template();
    let other = ResourceName::parse("collections/finance").unwrap();
    assert!(t.parse(&other).is_none(), "too few segments");

    let deeper = ResourceName::parse("collections/f/points/d/extra").unwrap();
    assert!(t.parse(&deeper).is_none(), "too many segments");
}

#[test]
fn parse_rejects_a_mismatched_literal() {
    let t = point_template();
    let other = ResourceName::parse("buckets/finance/points/doc-1").unwrap();
    assert!(t.parse(&other).is_none());
}

#[test]
fn format_rejects_a_value_containing_a_separator() {
    let t = point_template();
    let err = t.format(&["finance", "doc/123"]).unwrap_err();
    assert!(matches!(err, Error::InvalidResourceName { .. }));
}

#[test]
fn format_rejects_too_few_values() {
    assert!(point_template().format(&["finance"]).is_err());
}

#[test]
fn compile_rejects_an_empty_placeholder() {
    assert!(Template::compile("collections/{}").is_err());
}
