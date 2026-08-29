//! Create, get, list and delete a point over real gRPC.
//!
//! This is the "done when" test for the document service: a caller that only
//! ever speaks gRPC can create a point, fetch it back byte-for-byte, see it
//! in a listing, delete it, and get a real not-found afterward — backed by a
//! `redb` file on disk, not an in-memory stub.

use telividb_buffers::protobuf::point::v1::points_client::PointsClient;
use telividb_buffers::protobuf::point::v1::{
    ContentRef, CreatePointRequest, DeletePointRequest, GetPointRequest, ListPointsRequest, Point,
};

/// Start a server on an ephemeral port, with a fresh data directory, and wait
/// for it to accept connections.
mod support;

use support::server::TestServer;

#[tokio::test]
async fn create_get_list_delete_round_trip_over_grpc() {
    let server = TestServer::start().await;
    let addr = server.addr();
    support::collections::declare(addr, "media", "text_bge", 4).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let created = client
        .create_point(CreatePointRequest {
            parent: "collections/media".to_owned(),
            point_id: "doc-1".to_owned(),
            point: Some(Point {
                name: String::new(),
                vectors: Vec::new(),
                span: None,
                content_ref: Some(ContentRef {
                    uri: "s3://bucket/key".to_owned(),
                    range_start: 0,
                    range_end: 0,
                    sha256: Vec::new().into(),
                    inline_text: "hello world".to_owned(),
                }),
            }),
        })
        .await
        .expect("create should succeed")
        .into_inner();
    assert_eq!(created.name, "collections/media/points/doc-1");
    assert_eq!(
        created.content_ref.as_ref().map(|c| c.inline_text.as_str()),
        Some("hello world")
    );

    let fetched = client
        .get_point(GetPointRequest {
            name: created.name.clone(),
            read_mask: None,
        })
        .await
        .expect("get should find what create wrote")
        .into_inner();
    assert_eq!(fetched, created);

    let listed = client
        .list_points(ListPointsRequest {
            parent: "collections/media".to_owned(),
            page_size: 0,
            page_token: String::new(),
            read_mask: None,
        })
        .await
        .expect("list should answer")
        .into_inner();
    assert_eq!(listed.points, vec![created.clone()]);

    client
        .delete_point(DeletePointRequest {
            name: created.name.clone(),
        })
        .await
        .expect("delete should succeed");

    let err = client
        .get_point(GetPointRequest {
            name: created.name,
            read_mask: None,
        })
        .await
        .expect_err("a deleted point must be gone");
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn creating_the_same_point_twice_is_refused() {
    let server = TestServer::start().await;
    let addr = server.addr();
    support::collections::declare(addr, "media", "text_bge", 4).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let request = || CreatePointRequest {
        parent: "collections/media".to_owned(),
        point_id: "doc-1".to_owned(),
        point: Some(Point::default()),
    };
    client
        .create_point(request())
        .await
        .expect("first create succeeds");

    let err = client
        .create_point(request())
        .await
        .expect_err("second create must be refused");
    assert_eq!(err.code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn a_point_carrying_vectors_is_accepted_and_echoes_them_back() {
    // Inverted deliberately in stage 2: this asserted a refusal while vectors
    // had nowhere to go. They now persist through the field's WAL, so the
    // refusal would be the bug.
    let server = TestServer::start().await;
    let addr = server.addr();
    support::collections::declare(addr, "media", "text_bge", 4).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let data: Vec<u8> = [1.0f32, 0.0, 0.0, 0.0]
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();

    let created = client
        .create_point(CreatePointRequest {
            parent: "collections/media".to_owned(),
            point_id: "with-vectors".to_owned(),
            point: Some(Point {
                vectors: vec![telividb_buffers::protobuf::point::v1::NamedVector {
                    text: String::new(),
                    field_id: "text_bge".to_owned(),
                    vector: Some(telividb_buffers::protobuf::point::v1::Vector {
                        data: data.into(),
                        dimensions: 4,
                    }),
                }],
                ..Default::default()
            }),
        })
        .await
        .expect("a point carrying vectors must be accepted")
        .into_inner();

    assert_eq!(created.vectors.len(), 1);
    assert_eq!(created.vectors[0].field_id, "text_bge");
}
