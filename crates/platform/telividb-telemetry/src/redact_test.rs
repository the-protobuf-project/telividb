use super::*;

#[test]
fn resource_token_is_stable() {
    assert_eq!(
        resource_token("collections/finance/points/doc-1"),
        resource_token("collections/finance/points/doc-1")
    );
}

#[test]
fn resource_token_differs_between_resources() {
    assert_ne!(resource_token("a/b"), resource_token("a/c"));
}

#[test]
fn resource_token_does_not_contain_the_name() {
    let token = resource_token("collections/finance/points/doc-1");
    assert!(!token.contains("finance"));
    assert!(!token.contains("doc-1"));
}

#[test]
fn vector_shape_carries_only_the_dimension() {
    let shape = vector_shape(&[0.9, -0.2, 0.44]);
    assert_eq!(shape.dim, 3);
    let rendered = shape.to_string();
    for leaked in ["0.9", "-0.2", "0.44"] {
        assert!(!rendered.contains(leaked), "leaked a component: {rendered}");
    }
}

#[test]
fn vaults_are_recognised() {
    assert!(is_vault("vault/therapy-notes"));
    assert!(is_vault("vaults/personal"));
    assert!(!is_vault("collections/finance"));
}

#[test]
fn vault_collections_collapse_to_a_placeholder() {
    // Not hashed: a stable token would still reveal that a distinct private
    // collection exists and how often it is queried.
    assert_eq!(collection_label("vault/therapy-notes"), "<vault>");
    assert_eq!(collection_label("vaults/x"), "<vault>");
    assert_eq!(
        collection_label("collections/finance"),
        "collections/finance"
    );
}
