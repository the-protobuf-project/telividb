use super::*;

#[test]
fn a_collection_name_is_its_bare_id_under_collections() {
    assert_eq!(collection("documents"), "collections/documents");
}

#[test]
fn a_point_name_nests_under_its_collection() {
    // The parent of this name must be exactly `collection()`'s output, or a
    // create and the search that follows it address different places.
    assert_eq!(
        point("documents", "doc-1"),
        "collections/documents/points/doc-1"
    );
    assert!(point("documents", "doc-1").starts_with(&collection("documents")));
}

#[test]
fn an_id_round_trips_through_a_full_name() {
    // What the server returns is a full name; what a caller passes is an id.
    // If these disagree, every follow-up call after a search fails.
    let name = point("documents", "doc-1");
    assert_eq!(id_of(&name), "doc-1");
}

#[test]
fn a_bare_id_survives_extraction_unchanged() {
    // Guards the empty-string result a naive `split('/').last()` would give
    // for a name that is already an id.
    assert_eq!(id_of("doc-1"), "doc-1");
}
