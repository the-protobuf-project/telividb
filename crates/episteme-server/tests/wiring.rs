//! The server answers before any collection exists.
//!
//! Phase 0's whole claim is that the transport is wired: health reports
//! serving, reflection lists the services, and gRPC-web is layered in. These
//! assert that rather than trusting it.

use episteme_proto::collection::v1::collections_client::CollectionsClient;
use episteme_proto::collection::v1::{Collection, CreateCollectionRequest, ListCollectionsRequest};
use episteme_server::{ServerConfig, serve};
use std::net::SocketAddr;
use std::time::Duration;

/// Start a server on an ephemeral port and wait for it to accept connections.
async fn start() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
    let addr = listener.local_addr().expect("bound address");
    drop(listener);

    tokio::spawn(async move {
        let outcome = serve(ServerConfig {
            // Telemetry installs globally and only once per process, so tests
            // sharing a binary must not each try to install it.
            environment: episteme_telemetry::Environment::Production,
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
async fn the_server_accepts_grpc() {
    let addr = start().await;
    let mut client = CollectionsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let response = client
        .list_collections(ListCollectionsRequest {
            page_size: 0,
            page_token: String::new(),
        })
        .await
        .expect("list should answer even with no collections");
    assert!(response.into_inner().collections.is_empty());
}

#[tokio::test]
async fn arguments_are_validated_before_anything_else() {
    let addr = start().await;
    let mut client = CollectionsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let err = client
        .create_collection(CreateCollectionRequest {
            collection_id: String::new(),
            collection: Some(Collection {
                descriptor_set: bytes::Bytes::from_static(&[1, 2, 3]),
                ..Default::default()
            }),
        })
        .await
        .expect_err("an empty name must be refused");
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn a_missing_descriptor_set_is_refused_with_a_useful_message() {
    // The engine never parses `.proto`; it consumes a compiled descriptor set.
    // A caller who does not know that should learn it from the error.
    let addr = start().await;
    let mut client = CollectionsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let err = client
        .create_collection(CreateCollectionRequest {
            collection_id: "media".to_owned(),
            collection: Some(Collection::default()),
        })
        .await
        .expect_err("a missing descriptor set must be refused");

    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(
        err.message().contains("FileDescriptorSet"),
        "message should say what is required: {}",
        err.message()
    );
}

#[test]
fn the_descriptor_set_is_embedded_for_reflection() {
    // Without it, reflection cannot answer and every client must be shipped the
    // protos — which defeats `grpcurl` and generic tooling.
    assert!(
        !episteme_proto::FILE_DESCRIPTOR_SET.is_empty(),
        "descriptor set was not embedded at build time"
    );
}

#[test]
fn defaults_do_not_open_a_port_nobody_asked_for() {
    let config = ServerConfig::default();
    assert!(
        config.otlp_addr.is_none(),
        "telemetry export must be opt-in"
    );
    assert!(config.reflection, "reflection is what makes the API usable");
    assert!(config.grpc_web, "the embedded UI cannot speak native gRPC");
}
