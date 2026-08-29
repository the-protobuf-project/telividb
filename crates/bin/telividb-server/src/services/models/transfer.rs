//! Moving a model's bytes, and making it resident.
//!
//! Split from `install.rs` because that file is the RPC surface — accept,
//! report, cancel — and this is the work. The two change for different reasons:
//! a new field on the resource touches the first, and a change in how a model
//! is fetched or loaded touches only this.

use telividb_buffers::protobuf::models::v1::InstallationState;
use telividb_models::{CatalogEntry, HttpFetcher, ModelStore};
use telividb_telemetry::logger;

/// The transfer itself, on a blocking thread.
pub(super) fn install(
    store: &ModelStore,
    entry: &CatalogEntry,
    name: &str,
    installs: &super::Registry,
    embeddings: &crate::services::vector::Embeddings,
) -> Result<(), telividb_models::Error> {
    logger::info!("model install started").with_data(&serde_json::json!({
        "telividb.model.id": entry.id,
        "telividb.model.bytes": entry.size_bytes,
    }));

    let fetcher = HttpFetcher::new()?;
    let path = store.install(entry, &fetcher, &mut |written| {
        let Ok(mut guard) = installs.lock() else {
            return false;
        };
        let Some(record) = guard.get_mut(name) else {
            return false;
        };
        // A delete sets `Cancelled` while this runs; seeing it here is how
        // the transfer learns to stop.
        if record.state == InstallationState::Cancelled as i32 {
            return false;
        }
        record.state = InstallationState::Downloading as i32;
        record.progress_bytes = written as i64;
        true
    })?;

    // Load it now rather than at the next start. Reading several hundred
    // megabytes of weights takes seconds, so this happens here — on the
    // blocking thread that just finished the download — and the installation
    // stays `DOWNLOADING` until it is done. Reporting success before the model
    // could answer a query would be the same lie as the restart notice this
    // replaces.
    if let Err(e) = embeddings.install(&path, &entry.id) {
        logger::warn!("a model installed but could not be loaded").with_data(&serde_json::json!({
            "telividb.model.id": entry.id,
            "error": e.to_string(),
        }));
    }

    logger::info!("model resident").with_data(&serde_json::json!({
        "telividb.model.id": entry.id,
    }));
    Ok(())
}
