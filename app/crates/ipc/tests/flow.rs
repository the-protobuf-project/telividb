//! The first-run flow, against a real engine.
//!
//! Everything the window does on a fresh install, minus the window: start an
//! engine on an empty directory, create a collection from a shipped preset,
//! and read it back. What this proves is the part that can actually be broken
//! — that the preset's descriptor set is one the engine accepts, and that the
//! app's own client reaches its own server.
//!
//! Import is deliberately absent. It sends text, and the server refuses text
//! without an embedding model; a test that skipped when none was present would
//! pass by not running. The refusal itself is asserted below instead, because
//! that is what a fresh install actually meets.

use std::net::SocketAddr;
use std::path::PathBuf;
use telividb_desktop_engine::Engine;
use telividb_desktop_ipc::presets::{PRESETS, to_new_collection};

/// An engine on a fresh directory and an unused port.
async fn engine(tag: &str) -> (Engine, PathBuf) {
    let dir = std::env::temp_dir().join(format!("telividb-flow-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
    let addr: SocketAddr = listener.local_addr().expect("bound address");
    drop(listener);

    let engine = Engine::start(dir.clone(), addr, None)
        .await
        .expect("the engine starts on an empty directory");
    (engine, dir)
}

#[tokio::test]
async fn a_fresh_install_can_create_a_collection_from_a_preset() {
    let (engine, dir) = engine("create").await;
    let mut client = engine.client();

    // The preset's descriptor set is compiled bytes committed with the app.
    // If they were not a real `FileDescriptorSet`, this is where it shows.
    let spec = to_new_collection("notes", "my-notes", None).expect("notes is a preset");
    // The client returns the id rather than the full resource name — that is
    // what every other method on the handle takes, so returning a name here
    // would make the one thing a caller has to reformat before using.
    let id = client
        .create_collection(spec)
        .await
        .expect("the engine accepts the preset's descriptor set");
    assert_eq!(id, "my-notes");

    let listed = client.list_collections().await.expect("list");
    assert!(listed.iter().any(|id| id == "my-notes"));

    engine.shutdown().await;
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn every_shipped_preset_is_one_the_engine_accepts() {
    let (engine, dir) = engine("presets").await;
    let mut client = engine.client();

    // A preset that compiled but that the engine refuses would be a broken
    // choice in a picker — visible, selectable, and failing only on use.
    for preset in PRESETS {
        let spec = to_new_collection(preset.id, preset.id, None).expect("a shipped preset");
        client
            .create_collection(spec)
            .await
            .unwrap_or_else(|e| panic!("{} was refused: {e}", preset.id));
    }

    engine.shutdown().await;
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn text_is_refused_when_no_model_is_loaded() {
    let (engine, dir) = engine("nomodel").await;
    let mut client = engine.client();
    client
        .create_collection(to_new_collection("notes", "notes", None).expect("a preset"))
        .await
        .expect("create");

    // What a fresh install meets. The window asks for capabilities before
    // offering an import precisely so this refusal is never the first a person
    // hears of it — but the refusal has to be real for that warning to be true.
    let refused = client
        .collection("notes")
        .add_text("n-1", "text", "the merger discussion")
        .await
        .expect_err("text needs a model");
    let message = refused.to_string();
    assert!(
        message.contains("model"),
        "the refusal should name what is missing: {message}"
    );

    engine.shutdown().await;
    let _ = std::fs::remove_dir_all(&dir);
}
