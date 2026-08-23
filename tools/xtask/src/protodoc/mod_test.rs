use super::*;

#[test]
fn recognises_version_directories() {
    for name in ["v1", "v2", "v1beta1", "v2alpha"] {
        assert!(is_version_dir(name), "{name} should be a version");
    }
}

#[test]
fn rejects_names_that_merely_start_with_v() {
    for name in ["vectors", "v", "shared", ""] {
        assert!(!is_version_dir(name), "{name} should not be a version");
    }
}

#[test]
fn titles_module_names() {
    assert_eq!(title_case("collection"), "Collection");
    assert_eq!(title_case(""), "");
}
