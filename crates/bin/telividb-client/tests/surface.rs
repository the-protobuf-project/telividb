//! Reading points back, and what the server does not implement yet.
//!
//! The refusals are pinned deliberately. A client that quietly swallowed an
//! `Unimplemented` would make an unbuilt feature look like a working one, and
//! the test failing later is the signal to wire the real thing up.

mod common;

use common::{collection, connected, start};
use telividb_client::Client;

#[tokio::test]
async fn the_collection_catalogue_reports_that_it_is_not_implemented_yet() {
    // Pinned deliberately. `CreateCollection` is blocked on the `telividb.v1`
    // schema vocabulary, and a client that quietly swallowed the refusal would
    // make it look like collections were being created. When the catalogue
    // lands this test fails, which is the reminder to write the real one.
    let (mut client, _dir) = connected().await;

    match client
        .create_collection("anything", telividb_proto::FILE_DESCRIPTOR_SET.to_vec())
        .await
    {
        Err(telividb_client::Error::Server { code, .. }) => {
            assert_eq!(code, tonic::Code::Unimplemented);
        }
        other => panic!("expected Unimplemented from the catalogue, got {other:?}"),
    }
}

#[tokio::test]
async fn a_missing_endpoint_scheme_is_reported_as_such() {
    // `tonic` reports a bare host:port as an opaque URI error, which says
    // nothing about what to fix.
    match Client::connect("127.0.0.1:7700").await {
        Err(telividb_client::Error::InvalidArgument { message }) => {
            assert!(message.contains("scheme"), "got {message}");
        }
        other => panic!("expected an InvalidArgument about the scheme, got {other:?}"),
    }
}

#[tokio::test]
async fn the_batch_rpc_is_still_unimplemented() {
    // Pins why `insert_many` issues one request per point. When this fails,
    // the server has grown the batch RPC and `insert_many` should switch to
    // it — which is a change in round trips, not in behaviour.
    use telividb_proto::point::v1 as wire;
    use telividb_proto::point::v1::points_client::PointsClient;

    let (addr, _dir) = start().await;
    let mut raw = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let status = raw
        .batch_create_points(wire::BatchCreatePointsRequest {
            parent: "collections/anything".to_owned(),
            requests: Vec::new(),
        })
        .await
        .expect_err("the batch RPC should still refuse");

    assert_eq!(status.code(), tonic::Code::Unimplemented);
}

#[tokio::test]
async fn a_written_point_reads_back_with_its_identity_and_text() {
    let (client, _dir) = connected().await;
    let mut docs = collection(&client, "readback");

    docs.insert_with_text("doc-1", "text", &[1.5, -2.5], "the cat sat")
        .await
        .expect("insert");

    let record = docs.get("doc-1").await.expect("get").expect("present");
    assert_eq!(record.name, "doc-1");
    assert_eq!(record.text.as_deref(), Some("the cat sat"));
}

#[tokio::test]
async fn get_does_not_hand_back_raw_vectors() {
    // Pinned because it is a design decision, not an oversight. Vectors live
    // in the columnar field rather than the point's metadata record, and
    // reading them back is a permission scope of its own — a plain `GetPoint`
    // handing them out would give every reader `read_vector` for free.
    //
    // If this starts failing, the server has grown a `read_mask` path for
    // vectors; `Record::vectors` is already shaped to carry them.
    let (client, _dir) = connected().await;
    let mut docs = collection(&client, "novectors");

    docs.insert("doc-1", "text", &[1.5, -2.5])
        .await
        .expect("insert");

    let record = docs.get("doc-1").await.expect("get").expect("present");
    assert!(
        record.vectors.is_empty(),
        "raw vectors came back from a plain get: {:?}",
        record.vectors
    );
}

#[tokio::test]
async fn getting_a_point_that_was_never_written_is_none_not_an_error() {
    let (client, _dir) = connected().await;
    let mut docs = collection(&client, "absent");

    assert_eq!(docs.get("never-written").await.expect("get"), None);
}

#[tokio::test]
async fn listing_returns_every_written_point() {
    let (client, _dir) = connected().await;
    let mut docs = collection(&client, "listing");

    for i in 0..3 {
        docs.insert(&format!("doc-{i}"), "text", &[i as f32])
            .await
            .expect("insert");
    }

    let mut names: Vec<String> = docs
        .list()
        .await
        .expect("list")
        .into_iter()
        .map(|r| r.name)
        .collect();
    names.sort();

    assert_eq!(names, vec!["doc-0", "doc-1", "doc-2"]);
}
