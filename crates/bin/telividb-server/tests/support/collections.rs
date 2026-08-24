//! Declaring a collection, which every point test now needs.
//!
//! Points cannot be written to a collection that does not exist — the server
//! refuses rather than creating one as a side effect of the first write. That
//! is the structural rule these fixtures exist to satisfy, and satisfying it
//! here means each test states only what is interesting about *its* case.

use std::net::SocketAddr;
use telividb_proto::collection::v1::collections_client::CollectionsClient;
use telividb_proto::collection::v1::{Collection, CreateCollectionRequest, Metric, VectorField};

/// Create `id` declaring one field of `dim` dimensions under cosine.
///
/// Panics on failure: a fixture that quietly failed would leave every test
/// after it reporting the wrong reason.
pub async fn declare(addr: SocketAddr, id: &str, field: &str, dim: i32) {
    let mut client = CollectionsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect to the collections service");

    let outcome = client
        .create_collection(CreateCollectionRequest {
            collection_id: id.to_owned(),
            collection: Some(Collection {
                name: format!("collections/{id}"),
                // The engine never parses `.proto`; it consumes compiled
                // bytes. This workspace's own set is a real one.
                descriptor_set: telividb_proto::FILE_DESCRIPTOR_SET.to_vec().into(),
                vector_fields: vec![VectorField {
                    field_id: field.to_owned(),
                    dimensions: dim,
                    metric: Metric::Cosine as i32,
                    ..Default::default()
                }],
                ..Default::default()
            }),
        })
        .await;

    // `AlreadyExists` is fine: a restart test starts a second server over the
    // same data directory, where the catalogue already holds this collection.
    match outcome {
        Ok(_) => {}
        Err(status) if status.code() == tonic::Code::AlreadyExists => {}
        Err(status) => panic!("create collection: {status}"),
    }
}
