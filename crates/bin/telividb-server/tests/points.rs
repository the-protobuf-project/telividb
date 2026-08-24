//! Create, get, list and delete a point over real gRPC.
//!
//! This is the "done when" test for the document service: a caller that only
//! ever speaks gRPC can create a point, fetch it back byte-for-byte, see it
//! in a listing, delete it, and get a real not-found afterward — backed by a
//! `redb` file on disk, not an in-memory stub.

use std::net::SocketAddr;
use std::time::Duration;
use telividb_proto::point::v1::points_client::PointsClient;
use telividb_proto::point::v1::{
    ContentRef, CreatePointRequest, DeletePointRequest, GetPointRequest, ListPointsRequest, Point,
};
use telividb_server::{ServerConfig, serve};

/// Start a server on an ephemeral port, with a fresh data directory, and wait
/// for it to accept connections.
async fn start() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
    let addr = listener.local_addr().expect("bound address");
    drop(listener);

    let data_dir = tempfile::tempdir().expect("temp data dir").keep();

    tokio::spawn(async move {
        let outcome = serve(ServerConfig {
            // Telemetry installs globally and only once per process, so tests
            // sharing a binary must not each try to install it.
            environment: telividb_telemetry::Environment::Production,
            data_dir,
            ..ServerConfig::at(addr)
        })
        .await;
        if let Err(e) = outcome {
            eprintln!("SERVE FAILED: {e}");
        }
    });

    for _ in 0..100 {
        if std::net::TcpStream::connect(addr).is_ok() {
            return addr;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("server did not start on {addr}");
}

#[tokio::test]
async fn create_get_list_delete_round_trip_over_grpc() {
    let addr = start().await;
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
    let addr = start().await;
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
async fn named_vectors_are_refused_rather_than_silently_dropped() {
    let addr = start().await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let err = client
        .create_point(CreatePointRequest {
            parent: "collections/media".to_owned(),
            point_id: "doc-1".to_owned(),
            point: Some(Point {
                vectors: vec![telividb_proto::point::v1::NamedVector {
                    field_id: "text_bge".to_owned(),
                    vector: None,
                }],
                ..Default::default()
            }),
        })
        .await
        .expect_err("a point carrying vectors must be refused, not silently accepted");
    assert_eq!(err.code(), tonic::Code::Unimplemented);
}
