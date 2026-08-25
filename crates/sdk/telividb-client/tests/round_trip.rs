//! Writing and searching, against a real server.
//!
//! Every one of these could pass against a mock and still be wrong. What the
//! client actually does is agree with a server about a wire format — resource
//! names, the byte layout of a vector, which field carries the text — and a
//! mock agrees with whatever the client already does. So these start a real
//! server and talk to it.

mod common;

use common::{collection, connected};

#[tokio::test]
async fn a_vector_written_through_the_sdk_is_found_by_searching_for_it() {
    // The end-to-end claim: the client's encoding, the server's decoding, its
    // storage, and its search all agree. A disagreement anywhere shows up as
    // an empty result rather than an error, which is why this asserts on a
    // hit rather than on a status code.
    let (mut client, _dir) = connected().await;
    let mut docs = collection(&mut client, "roundtrip", 3).await;

    docs.insert("doc-1", "text", &[1.0, 0.0, 0.0])
        .await
        .expect("insert");

    let found = docs
        .search("text", &[1.0, 0.0, 0.0], 5)
        .await
        .expect("search");

    assert_eq!(found.len(), 1, "the written vector should come back");
    assert_eq!(found.hits()[0].name, "doc-1");
}

#[tokio::test]
async fn search_ranks_the_nearer_vector_first() {
    // Guards the case where everything round-trips but the bytes were
    // reinterpreted: a transposed or mis-decoded vector still returns results,
    // in the wrong order.
    let (mut client, _dir) = connected().await;
    let mut docs = collection(&mut client, "ranking", 2).await;

    docs.insert("near", "text", &[1.0, 0.0])
        .await
        .expect("near");
    docs.insert("far", "text", &[0.0, 1.0]).await.expect("far");

    let found = docs.search("text", &[0.9, 0.1], 2).await.expect("search");

    assert_eq!(found.len(), 2);
    assert_eq!(found.hits()[0].name, "near", "got {:?}", found.hits());
    assert!(found.hits()[0].score > found.hits()[1].score);
}

#[tokio::test]
async fn text_stored_with_a_vector_comes_back_on_the_hit() {
    // Without this a result is a bare id, and the caller has to resolve it
    // against their own storage before it means anything.
    let (mut client, _dir) = connected().await;
    let mut docs = collection(&mut client, "withtext", 2).await;

    docs.insert_with_text("doc-1", "text", &[1.0, 0.0], "the cat sat")
        .await
        .expect("insert");

    let found = docs.search("text", &[1.0, 0.0], 1).await.expect("search");
    assert_eq!(found.hits()[0].text.as_deref(), Some("the cat sat"));
}

#[tokio::test]
async fn inserting_many_stores_every_point() {
    let (mut client, _dir) = connected().await;
    let mut docs = collection(&mut client, "batch", 2).await;

    let entries: Vec<(String, Vec<f32>, String)> = (0..5)
        .map(|i| {
            (
                format!("doc-{i}"),
                vec![i as f32, 1.0],
                format!("document {i}"),
            )
        })
        .collect();

    let created = docs
        .insert_many("text", &entries)
        .await
        .expect("insert many");
    assert_eq!(created.len(), 5);

    let found = docs.search("text", &[4.0, 1.0], 5).await.expect("search");
    assert_eq!(found.len(), 5);
    assert_eq!(found.hits()[0].name, "doc-4", "got {:?}", found.hits());
}

#[tokio::test]
async fn a_single_node_search_reports_itself_complete() {
    // Rules 27 and 49: a caller must be able to tell "nothing matched" from
    // "nothing you can see matched". With nothing locked, this is `true`.
    let (mut client, _dir) = connected().await;
    let mut docs = collection(&mut client, "complete", 1).await;

    docs.insert("doc-1", "text", &[1.0]).await.expect("insert");
    let found = docs.search("text", &[1.0], 1).await.expect("search");

    assert!(found.is_complete());
    assert!(found.locked_vaults().is_empty());
}

#[tokio::test]
async fn a_search_over_an_empty_collection_returns_nothing_rather_than_failing() {
    // A collection may simply not carry the field yet. That is an ordinary
    // outcome, not an error.
    let (mut client, _dir) = connected().await;
    let mut docs = collection(&mut client, "empty", 2).await;

    let found = docs.search("text", &[1.0, 0.0], 5).await.expect("search");
    assert!(found.is_empty());
    assert!(found.is_complete());
}

#[tokio::test]
async fn a_deleted_point_stops_being_found() {
    let (mut client, _dir) = connected().await;
    let mut docs = collection(&mut client, "deletion", 2).await;

    docs.insert("doc-1", "text", &[1.0, 0.0])
        .await
        .expect("insert");
    docs.delete("doc-1").await.expect("delete");

    let found = docs.search("text", &[1.0, 0.0], 5).await.expect("search");
    assert!(
        found.hits().iter().all(|h| h.name != "doc-1"),
        "a deleted point came back: {:?}",
        found.hits()
    );
}
