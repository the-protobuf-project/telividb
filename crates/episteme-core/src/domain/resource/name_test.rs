use super::*;

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

#[test]
fn rejects_malformed_names() {
    for bad in ["", "/collections/a", "collections/a/", "collections//a"] {
        assert!(ResourceName::parse(bad).is_err(), "should reject {bad:?}");
    }
}

#[test]
fn accepts_punctuation_within_a_segment() {
    assert_eq!(name("collections/a/points/doc-1.2_3").leaf(), "doc-1.2_3");
}

#[test]
fn parent_strips_collection_and_id() {
    let n = name("collections/finance/points/doc-123");
    assert_eq!(n.parent().unwrap().as_str(), "collections/finance");
}

#[test]
fn parent_of_a_root_resource_is_none() {
    assert!(name("collections").parent().is_none());
}

#[test]
fn single_star_matches_exactly_one_segment() {
    let n = name("collections/finance/points/doc-123");
    assert!(n.matches("collections/finance/points/*"));
    assert!(n.matches("collections/*/points/*"));
    assert!(!n.matches("collections/other/points/*"));
    assert!(
        !n.matches("collections/*/points"),
        "must not match a prefix"
    );
}

#[test]
fn double_star_matches_the_remainder() {
    let n = name("collections/finance/points/doc-123");
    assert!(n.matches("collections/**"));
    assert!(n.matches("collections/finance/**"));
    assert!(
        !name("collections").matches("collections/**"),
        "needs a remainder"
    );
}

#[test]
fn exact_pattern_matches_itself() {
    let n = name("collections/finance");
    assert!(n.matches("collections/finance"));
    assert!(!n.matches("collections/finance/points/x"));
}
