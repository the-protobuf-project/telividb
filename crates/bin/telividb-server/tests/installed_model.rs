//! A model installed into the data directory is found and used, with no
//! configuration at all.
//!
//! This covers the failure that made the product look broken: installing a model
//! from the catalog appeared to succeed and changed nothing, because the engine
//! consulted `TELIVIDB_MODEL` and never looked in its own model directory. A
//! restart did not help either — there was nothing to find it with.
//!
//! Skipped when the model is absent (639 MiB, not committed), and the skip says
//! so out loud rather than passing quietly.

mod support;

use std::path::PathBuf;
use support::server::TestServer;
use telividb_buffers::protobuf::point::v1::points_client::PointsClient;
use telividb_buffers::protobuf::point::v1::{
    CreatePointRequest, NamedVector, Point, SearchPointsRequest,
};

/// The catalog id whose file the store looks for, and the width it produces.
const MODEL_ID: &str = "qwen3-embedding-0.6b";

/// A downloaded copy to install into the server's data directory.
fn source_model() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/models/gguf/text/Qwen3-Embedding-0.6B-Q8_0.gguf");
    path.exists().then_some(path)
}

#[tokio::test]
async fn a_model_in_the_data_directory_is_loaded_without_being_configured() {
    let Some(source) = source_model() else {
        eprintln!("SKIPPED: Qwen3-Embedding-0.6B-Q8_0.gguf is not present");
        return;
    };

    // Place the model exactly where an install would leave it, and start a
    // server told nothing about it.
    let server = TestServer::start_with(|data_dir| {
        let models = data_dir.join("models");
        std::fs::create_dir_all(&models).expect("models dir");
        std::fs::copy(&source, models.join(format!("{MODEL_ID}.gguf"))).expect("place the model");
    })
    .await;

    let mut points = PointsClient::connect(server.url()).await.expect("connect");
    support::collections::declare(server.addr(), "notes", "text", 1024).await;

    // The model loads after the listener binds, so the server is reachable
    // before it can embed. That ordering is deliberate — it is what stops a
    // slow load from delaying startup — and it means a caller may arrive first.
    let mut ready = false;
    for _ in 0..600 {
        let probe = points
            .create_point(CreatePointRequest {
                parent: "collections/notes".to_owned(),
                point_id: "probe".to_owned(),
                point: Some(Point {
                    vectors: vec![NamedVector {
                        field_id: "text".to_owned(),
                        text: "a warm-up sentence".to_owned(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            })
            .await;
        if probe.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(ready, "the installed model never became resident");

    // The probe is a stored point like any other, so it would compete in the
    // ranking below — and it did, coming first.
    points
        .delete_point(telividb_buffers::protobuf::point::v1::DeletePointRequest {
            name: "collections/notes/points/probe".to_owned(),
        })
        .await
        .expect("removing the readiness probe");

    // Plain English in — no vector, no client-side model.
    for (id, text) in [
        ("cat", "The cat sat quietly on the mat by the window."),
        (
            "rust",
            "Rust guarantees memory safety without a garbage collector.",
        ),
        (
            "rain",
            "Heavy rain is expected across the region this weekend.",
        ),
    ] {
        points
            .create_point(CreatePointRequest {
                parent: "collections/notes".to_owned(),
                point_id: id.to_owned(),
                point: Some(Point {
                    vectors: vec![NamedVector {
                        field_id: "text".to_owned(),
                        text: text.to_owned(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            })
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "storing {id:?} as text failed, which means the installed \
                     model was not loaded: {e}"
                )
            });
    }

    // Plain English out.
    let found = points
        .search_points(SearchPointsRequest {
            parent: "collections/notes".to_owned(),
            field_id: "text".to_owned(),
            query_text: "a pet resting indoors".to_owned(),
            page_size: 3,
            ..Default::default()
        })
        .await
        .expect("searching by text")
        .into_inner();

    let ranked: Vec<&str> = found
        .results
        .iter()
        .filter_map(|r| r.point.as_ref())
        .map(|p| p.name.rsplit('/').next().unwrap_or(""))
        .collect();
    assert_eq!(
        ranked.first().copied(),
        Some("cat"),
        "the question is about a pet indoors; ranking was {ranked:?}"
    );
}
