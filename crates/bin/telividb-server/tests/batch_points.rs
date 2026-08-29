//! `BatchCreatePoints` over real gRPC.
//!
//! The reason this method exists is the round trip: importing a CSV one row at
//! a time is one request per row before any work happens. So the test that
//! matters is that a batch writes every point and that a failure says enough
//! for a caller to resubmit — not that it is faster, which is not observable
//! from here.

use telividb_buffers::protobuf::point::v1::points_client::PointsClient;
use telividb_buffers::protobuf::point::v1::{
    BatchCreatePointsRequest, CreatePointRequest, ListPointsRequest,
};

mod support;

use support::server::TestServer;

/// One create request for `id`, carrying a vector of the declared width.
fn item(id: &str, values: &[f32]) -> CreatePointRequest {
    CreatePointRequest {
        parent: String::new(),
        point_id: id.to_owned(),
        point: Some(support::vectors::point_with(values)),
    }
}

#[tokio::test]
async fn a_batch_writes_every_point_it_was_given() {
    let server = TestServer::start().await;
    let addr = server.addr();
    support::collections::declare(addr, "media", "text_bge", 4).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let created = client
        .batch_create_points(BatchCreatePointsRequest {
            parent: "collections/media".to_owned(),
            requests: vec![
                item("row-1", &[1.0, 0.0, 0.0, 0.0]),
                item("row-2", &[0.0, 1.0, 0.0, 0.0]),
                item("row-3", &[0.0, 0.0, 1.0, 0.0]),
            ],
        })
        .await
        .expect("batch create should succeed")
        .into_inner();

    assert_eq!(created.points.len(), 3);
    assert_eq!(created.points[0].name, "collections/media/points/row-1");

    // Written, not merely reported: the listing is the independent check that
    // the batch reached storage rather than only the response.
    let listed = client
        .list_points(ListPointsRequest {
            parent: "collections/media".to_owned(),
            page_size: 0,
            page_token: String::new(),
            read_mask: None,
        })
        .await
        .expect("list")
        .into_inner();
    assert_eq!(listed.points.len(), 3);
}

#[tokio::test]
async fn an_empty_batch_is_refused_rather_than_treated_as_a_no_op() {
    let server = TestServer::start().await;
    let addr = server.addr();
    support::collections::declare(addr, "media", "text_bge", 4).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let status = client
        .batch_create_points(BatchCreatePointsRequest {
            parent: "collections/media".to_owned(),
            requests: Vec::new(),
        })
        .await
        .expect_err("an empty batch is a mistake, not a no-op");
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn a_failure_names_the_item_and_how_far_the_batch_got() {
    let server = TestServer::start().await;
    let addr = server.addr();
    support::collections::declare(addr, "media", "text_bge", 4).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    // The third row carries the wrong width for the declared field. Without
    // the index and the count, a caller could not tell which row to fix or
    // which ones are already written — so retrying would duplicate the first
    // two.
    let status = client
        .batch_create_points(BatchCreatePointsRequest {
            parent: "collections/media".to_owned(),
            requests: vec![
                item("row-1", &[1.0, 0.0, 0.0, 0.0]),
                item("row-2", &[0.0, 1.0, 0.0, 0.0]),
                item("row-3", &[0.0, 0.0, 1.0]),
            ],
        })
        .await
        .expect_err("a mismatched width should be refused");

    let message = status.message();
    assert!(message.contains("requests[2]"), "no item index: {message}");
    assert!(
        message.contains("2 point(s)"),
        "no written count: {message}"
    );
}

#[tokio::test]
async fn an_item_naming_another_collection_is_refused() {
    let server = TestServer::start().await;
    let addr = server.addr();
    support::collections::declare(addr, "media", "text_bge", 4).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let mut stray = item("row-1", &[1.0, 0.0, 0.0, 0.0]);
    stray.parent = "collections/elsewhere".to_owned();

    // Silently rewriting it would put the point somewhere the caller did not
    // ask for; ignoring the field would make it meaningless.
    let status = client
        .batch_create_points(BatchCreatePointsRequest {
            parent: "collections/media".to_owned(),
            requests: vec![stray],
        })
        .await
        .expect_err("a batch writes to one collection");
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}
