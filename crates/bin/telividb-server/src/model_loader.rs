//! Choosing a model and loading it into the running server.
//!
//! Separate from `serve.rs` because the two answer different questions: that
//! file decides what is served, and this decides what is resident. It also runs
//! at a different time — *after* the listener binds, deliberately.
//!
//! # Why loading happens after the port is open
//!
//! Reading a model takes real time: 47 s for a 639 MB one on an M3 Max, most of
//! it hashing the file and uploading weights to the device. Doing that before
//! binding made the port appear that much later, and the desktop app — which
//! waits a few seconds for its engine — gave up and shut the whole thing down.
//! Startup blocked on work no caller was waiting for.
//!
//! So the server serves vectors immediately, text requests are refused with a
//! reason until the model lands, and a caller that asks is told which model is
//! resident. That is the same state a freshly installed model passes through,
//! which means it is a state the rest of the system already handles.

use crate::services::Embeddings;
use telividb_telemetry::{fields, logger};

pub(crate) async fn load_model(
    configured: Option<std::path::PathBuf>,
    name: String,
    data_dir: std::path::PathBuf,
    embeddings: Embeddings,
) {
    // Blocking: reading and uploading weights is neither async nor quick, and
    // it must not sit on an executor thread (invariant 5).
    let _ = tokio::task::spawn_blocking(move || {
        load_model_blocking(configured, &name, &data_dir, &embeddings)
    })
    .await;
}

/// Choose a model and load it, reporting what happened either way.
fn load_model_blocking(
    configured: Option<std::path::PathBuf>,
    name: &str,
    data_dir: &std::path::Path,
    embeddings: &Embeddings,
) {
    // An explicitly configured model wins: someone who named a path meant that
    // file, and quietly preferring an installed one would ignore them.
    if let Some(path) = configured {
        match embeddings.install(&path, name) {
            Ok(()) => {
                logger::info!("loaded the configured model").with_data(&serde_json::json!({
                    fields::MODEL: name,
                }));
            }
            Err(e) => {
                logger::error!("the configured model could not be loaded")
                    .with_data(&serde_json::json!({ fields::MODEL: name, "error": e.to_string() }));
            }
        }
        return;
    }

    // Otherwise, load whatever the catalog says is installed. Without this an
    // installed model was never found at all — not on the next request, and not
    // after a restart either, because nothing but `TELIVIDB_MODEL` was ever
    // consulted. Installing one appeared to succeed and changed nothing.
    let store = telividb_models::ModelStore::new(data_dir.join("models"));
    let catalog = telividb_models::Catalog::builtin();
    let Some(entry) = store.resident_choice(&catalog) else {
        logger::info!("no embedding model configured").with_data(&serde_json::json!({
            fields::STRATEGY: "vectors-only",
        }));
        return;
    };

    let path = store.path_of(&entry.id);
    // Not fatal, unlike a configured path. A model that was installed and has
    // since become unreadable should leave a server that still serves vectors,
    // with the reason in the log — refusing to start would take the whole
    // window down over a file the person can simply install again.
    match embeddings.install(&path, &entry.id) {
        Ok(()) => {
            logger::info!("loaded an installed model").with_data(&serde_json::json!({
                fields::MODEL: entry.id,
            }));
        }
        Err(e) => {
            logger::warn!("an installed model could not be loaded")
                .with_data(&serde_json::json!({ fields::MODEL: entry.id, "error": e.to_string() }));
        }
    }
}
