//! Vectors, end to end over gRPC: write, search, and survive a restart.
//!
//! The "done when" test for the vector service — it exercises the whole Phase 1
//! lifecycle from outside the process (WAL append, searchable buffer, GPU
//! index, recovery) through nothing but the public API.

mod support;

use support::vectors::{create, search, start_at, started};
use telividb_proto::point::v1::points_client::PointsClient;

#[tokio::test]
async fn vectors_are_written_and_searched_over_grpc() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start_at(dir.path().to_path_buf()).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    // Three points whose similarity to [1,0,0,0] is unambiguous.
    client
        .create_point(create("a", &[1.0, 0.0, 0.0, 0.0]))
        .await
        .unwrap();
    client
        .create_point(create("b", &[0.0, 1.0, 0.0, 0.0]))
        .await
        .unwrap();
    client
        .create_point(create("c", &[0.9, 0.1, 0.0, 0.0]))
        .await
        .unwrap();

    let response = client
        .search_points(search(&[1.0, 0.0, 0.0, 0.0], 3))
        .await
        .expect("search should answer")
        .into_inner();

    let names: Vec<String> = response
        .results
        .iter()
        .filter_map(|r| r.point.as_ref().map(|p| p.name.clone()))
        .collect();

    assert_eq!(names.len(), 3, "every point should be a candidate");
    assert_eq!(
        names[0], "collections/media/points/a",
        "the identical vector must rank first"
    );
    assert!(
        response.complete,
        "single-node search always answers completely"
    );
}

#[tokio::test]
async fn a_just_written_vector_is_immediately_findable() {
    // The searchable-buffer guarantee: nothing is sealed at this size, so a
    // write that were only durable and not searchable would fail here.
    let dir = tempfile::tempdir().unwrap();
    let addr = start_at(dir.path().to_path_buf()).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    client
        .create_point(create("fresh", &[0.0, 0.0, 1.0, 0.0]))
        .await
        .unwrap();

    let found = client
        .search_points(search(&[0.0, 0.0, 1.0, 0.0], 1))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(found.results.len(), 1);
    assert_eq!(
        found.results[0].point.as_ref().unwrap().name,
        "collections/media/points/fresh"
    );
}

#[tokio::test]
async fn vectors_survive_a_restart_through_the_wal() {
    // The point of writing the log before the buffer. Nothing here is ever
    // sealed — the corpus is far under the threshold — so the only way these
    // vectors come back is WAL replay.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();

    // Stopped, not merely abandoned: redb holds an exclusive lock, so the
    // second server cannot open the collection until the first lets go.
    let (addr, first) = started(path.clone()).await;
    {
        let mut client = PointsClient::connect(format!("http://{addr}"))
            .await
            .expect("connect");
        client
            .create_point(create("persisted", &[1.0, 0.0, 0.0, 0.0]))
            .await
            .unwrap();
    }
    first.stop().await;

    // A second server over the same directory: a fresh process would behave
    // the same way, and this is what a test can observe.
    let addr = start_at(path).await;
    let mut client = PointsClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let found = client
        .search_points(search(&[1.0, 0.0, 0.0, 0.0], 5))
        .await
        .expect("search after restart")
        .into_inner();

    assert_eq!(
        found.results.len(),
        1,
        "the vector did not survive the restart"
    );
    assert_eq!(
        found.results[0].point.as_ref().unwrap().name,
        "collections/media/points/persisted"
    );
}
