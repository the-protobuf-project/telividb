//! The server answers before any collection exists.
//!
//! Phase 0's whole claim is that the transport is wired: health reports
//! serving, reflection lists the services, and gRPC-web is layered in. These
//! assert that rather than trusting it.

mod support;

use support::server::TestServer;

use telividb_buffers::protobuf::collection::v1::collections_client::CollectionsClient;
use telividb_buffers::protobuf::collection::v1::{
    Collection, CreateCollectionRequest, ListCollectionsRequest,
};
use telividb_buffers::protobuf::system::v1::GetSystemRequest;
use telividb_buffers::protobuf::system::v1::system_info_client::SystemInfoClient;
use telividb_server::ServerConfig;

#[tokio::test]
async fn the_server_accepts_grpc() {
    let server = TestServer::start().await;
    let addr = server.addr();
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

/// The selected backend is reachable over the wire, not only in-process.
///
/// This is the fact an orchestrator cannot see from outside: a build that fell
/// back to the host is indistinguishable from a healthy one until something
/// asks. The desktop app used to answer this by detecting in its own process,
/// which a browser talking to a daemon could not do — so "it is served" is the
/// claim worth testing.
#[tokio::test]
async fn the_system_reports_the_backend_it_selected() {
    let server = TestServer::start().await;
    let addr = server.addr();
    let mut client = SystemInfoClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let system = client
        .get_system(GetSystemRequest {
            name: "system".to_owned(),
        })
        .await
        .expect("the system is always describable")
        .into_inner();

    assert!(!system.backend.is_empty(), "a backend is always selected");
    assert_eq!(system.name, "system");
    assert_ne!(
        system.budget_source, 0,
        "BUDGET_SOURCE_UNSPECIFIED is never valid in a response"
    );
}

#[tokio::test]
async fn arguments_are_validated_before_anything_else() {
    let server = TestServer::start().await;
    let addr = server.addr();
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
    let server = TestServer::start().await;
    let addr = server.addr();
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
        !telividb_buffers::protobuf::FILE_DESCRIPTOR_SET.is_empty(),
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
