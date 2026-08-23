use super::*;

/// Wrap source lines the way the scanner sees them.
fn scan(source: &str) -> Vec<usize> {
    let lines: Vec<&str> = source.lines().collect();
    (0..lines.len())
        .filter(|&i| is_undocumented_item(&lines, i, Path::new("lib.rs")))
        .collect()
}

#[test]
fn an_undocumented_pub_function_is_reported() {
    // The gap the compiler leaves: `pub` inside a private module is not
    // publicly reachable, so `deny(missing_docs)` never sees it.
    assert_eq!(scan("pub fn nearest_centroid() -> usize { 0 }"), vec![0]);
}

#[test]
fn a_documented_pub_function_is_accepted() {
    assert!(scan("/// Index of the nearest centroid.\npub fn nearest() {}").is_empty());
}

#[test]
fn attributes_between_the_doc_and_the_item_are_stepped_over() {
    let source = "/// Something worth saying.\n#[inline]\n#[must_use]\npub fn f() {}";
    assert!(scan(source).is_empty(), "attributes broke the lookback");
}

#[test]
fn a_private_item_is_not_the_scanners_business() {
    assert!(scan("fn helper() {}").is_empty());
}

#[test]
fn restricted_visibility_is_not_treated_as_public() {
    // `pub(crate)` and `pub(super)` are internal wiring. Holding them to the
    // same bar would bury the real findings.
    assert!(scan("pub(crate) fn wiring() {}").is_empty());
    assert!(scan("pub(super) struct Inner;").is_empty());
}

#[test]
fn an_inline_module_needs_a_doc_comment_above_it() {
    // `pub mod foo;` is documented from inside `foo.rs`, but an inline module
    // has no file to carry a `//!`.
    assert_eq!(scan("pub mod v1 {\n}"), vec![0]);
    assert!(scan("/// Version 1 of the resource.\npub mod v1 {\n}").is_empty());
}

#[test]
fn every_item_kind_is_covered() {
    for kind in ITEM_KINDS {
        let source = format!("pub {kind} Thing");
        assert_eq!(scan(&source), vec![0], "{kind} was not detected");
    }
}
