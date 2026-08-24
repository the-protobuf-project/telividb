//! What the vector service refuses, and what it answers emptily.
//!
//! Separated from the happy path because these assert the *shape of failure* —
//! which is where a service usually drifts: an error that should be an empty
//! result, or a guess where there should be a refusal.

mod support;

use support::vectors::{DIM, FIELD, start_at, wire_vector};
use telividb_proto::point::v1::points_client::PointsClient;
use telividb_proto::point::v1::{SearchPointsRequest, Vector};

#[tokio::test]
async fn a_search_against_an_unknown_field_is_empty_not_an_error() {
    // A collection may simply not carry this field yet.
    let dir = tempfile::tempdir().unwrap();
    let addr = start_at(dir.path().to_path_buf()).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let found = client
        .search_points(SearchPointsRequest {
            parent: "collections/media".to_owned(),
            field_id: "never_written".to_owned(),
            query: Some(wire_vector(&[1.0, 0.0, 0.0, 0.0])),
            page_size: 5,
            ..Default::default()
        })
        .await
        .expect("an unknown field should answer, not fail")
        .into_inner();
    assert!(found.results.is_empty());
}

#[tokio::test]
async fn a_malformed_query_vector_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start_at(dir.path().to_path_buf()).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    // Declares four dimensions, carries one float.
    let err = client
        .search_points(SearchPointsRequest {
            parent: "collections/media".to_owned(),
            field_id: FIELD.to_owned(),
            query: Some(Vector {
                data: vec![0u8; 4].into(),
                dimensions: DIM as i32,
            }),
            page_size: 1,
            ..Default::default()
        })
        .await
        .expect_err("a length mismatch must be refused");
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn searching_without_a_field_is_refused() {
    // Each field has its own model and metric, so a query with no field named
    // cannot be meaningful — better to say so than to guess one.
    let dir = tempfile::tempdir().unwrap();
    let addr = start_at(dir.path().to_path_buf()).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let err = client
        .search_points(SearchPointsRequest {
            parent: "collections/media".to_owned(),
            field_id: String::new(),
            query: Some(wire_vector(&[1.0, 0.0, 0.0, 0.0])),
            page_size: 1,
            ..Default::default()
        })
        .await
        .expect_err("an empty field_id must be refused");
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}
