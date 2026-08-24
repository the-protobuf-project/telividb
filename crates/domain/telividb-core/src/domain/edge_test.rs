use super::*;
use crate::domain::ResourceName;

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

#[test]
fn new_carries_every_field_through() {
    let edge = Edge::new(
        name("collections/media/points/a"),
        name("collections/media/points/b"),
        "HAS_SHOT",
        0.5,
    );
    assert_eq!(edge.src, name("collections/media/points/a"));
    assert_eq!(edge.dst, name("collections/media/points/b"));
    assert_eq!(edge.edge_type, "HAS_SHOT");
    assert_eq!(edge.weight, 0.5);
}

#[test]
fn edge_type_accepts_a_string_or_a_literal() {
    let owned = Edge::new(name("a/1"), name("b/1"), String::from("MENTIONS"), 1.0);
    let literal = Edge::new(name("a/1"), name("b/1"), "MENTIONS", 1.0);
    assert_eq!(owned.edge_type, literal.edge_type);
}
