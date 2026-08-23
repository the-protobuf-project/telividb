use super::*;

fn modules() -> Vec<Module> {
    vec![
        Module {
            name: "Shared".to_owned(),
            package: "demo.shared.v1".to_owned(),
            dir: "demo/shared".to_owned(),
            ..Default::default()
        },
        Module {
            name: "Widget".to_owned(),
            package: "demo.widget.v1".to_owned(),
            dir: "demo/widget".to_owned(),
            imports: vec![
                "demo/shared/v1/types.proto".to_owned(),
                "google/protobuf/duration.proto".to_owned(),
            ],
            ..Default::default()
        },
    ]
}

#[test]
fn lists_every_module() {
    let out = root_readme(&modules(), "demo/");
    assert!(out.contains("[Shared](demo/shared/README.md)"));
    assert!(out.contains("[Widget](demo/widget/README.md)"));
}

#[test]
fn draws_local_edges_only() {
    // An arrow from every node to `google.protobuf` would be true and useless.
    let out = root_readme(&modules(), "demo/");
    assert!(out.contains("widget --> shared"));
    assert!(
        !out.contains("protobuf"),
        "well-known imports must not appear"
    );
}

#[test]
fn omits_the_graph_when_nothing_imports_anything() {
    let plain = vec![Module {
        name: "Solo".to_owned(),
        dir: "demo/solo".to_owned(),
        ..Default::default()
    }];
    assert!(!root_readme(&plain, "demo/").contains("mermaid"));
}

#[test]
fn a_module_does_not_import_itself() {
    let m = Module {
        dir: "demo/widget".to_owned(),
        imports: vec!["demo/widget/v1/widget.proto".to_owned()],
        ..Default::default()
    };
    assert!(m.local_imports("demo/").is_empty());
}

#[test]
fn node_ids_are_safe_for_mermaid() {
    // Punctuation in an identifier ends it early and produces a broken diagram.
    assert_eq!(sanitize("my-module.v1"), "my_module_v1");
}
