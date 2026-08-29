//! The models service: what is on offer, and installing it.
//!
//! Separate from the inference service, which is about models already resident
//! and the vectors they produce. This is about acquiring the files.
//!
//! # An install is a resource, not a call
//!
//! A model file is hundreds of megabytes, so a transfer outlives any request
//! deadline (invariant 10). Creating an installation returns immediately with a
//! handle; progress is read back by getting it, and deleting it cancels.
//!
//! # What a restart does, stated rather than discovered
//!
//! The registry of installations is in memory, so a restart forgets that an
//! install was ever running. What survives is the **partial file**, which is
//! what actually matters: installing again resumes from it rather than starting
//! over. So a restart loses the progress bar, never the progress.

mod catalog;
mod convert;
mod install;
mod service;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use telividb_buffers::protobuf::models::v1::ModelInstallation;
use telividb_models::{Catalog, ModelStore};
use tonic::Status;

/// The catalog, the model directory, and every installation this process knows
/// about.
#[derive(Clone)]
pub struct ModelsSvc {
    /// The compiled-in catalog. Cheap to clone and never changes at runtime.
    catalog: Catalog,
    /// Where model files live.
    store: ModelStore,
    /// Installations by resource name.
    ///
    /// A `Mutex` rather than a channel because every access is a short read or
    /// a field update — the transfer itself happens on a blocking thread and
    /// touches this only to report progress.
    installs: Registry,
}

impl ModelsSvc {
    /// Serve the catalog, storing models under `data_dir/models`.
    pub fn new(data_dir: &std::path::Path) -> Self {
        Self {
            catalog: Catalog::builtin(),
            store: ModelStore::new(data_dir.join("models")),
            installs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// The installation registry, or a message if a panic poisoned it.
    fn installs(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, ModelInstallation>>, Status> {
        self.installs
            .lock()
            .map_err(|_| Status::internal("the installation registry was poisoned by a panic"))
    }
}

/// Every installation this process knows about, keyed by resource name.
///
/// Shared with the blocking transfer threads, which report progress into it and
/// read the cancel flag back out.
type Registry = Arc<Mutex<HashMap<String, ModelInstallation>>>;

/// The resource name for a catalog model.
fn catalog_name(id: &str) -> String {
    format!("catalogModels/{id}")
}

/// The catalog id inside a `catalogModels/{id}` name.
///
/// Accepts a bare id too. A caller that has just read `id` from a listing and
/// passes it back should not be corrected on a formality.
fn catalog_id(name: &str) -> &str {
    name.strip_prefix("catalogModels/").unwrap_or(name)
}
