//! Opening and caching a collection's `redb` point store.
//!
//! Split from `point.rs` because it answers a different question: that file is
//! the service and its RPC surface, this is how a collection's metadata store
//! is found and kept open.
//!
//! The caching is not an optimisation. `redb` takes an **exclusive file
//! lock**, so two concurrent requests against one collection cannot each open
//! their own handle — the second fails with "Database already open".

use super::service::PointsSvc;
use crate::error::storage_status;
use std::path::PathBuf;
use std::sync::Arc;
use telividb_core::ResourceName;
use telividb_storage::RedbPointStore;
use tonic::Status;

impl PointsSvc {
    /// Path to the `redb` file for `collection`, e.g. `media` from
    /// `collections/media`.
    pub(super) fn store_path(&self, collection: &ResourceName) -> PathBuf {
        self.data_dir.join(collection.leaf()).join("points.redb")
    }

    /// The cached handle for `collection`, opening it on first use.
    pub(super) fn store(&self, collection: &ResourceName) -> Result<Arc<RedbPointStore>, Status> {
        let key = collection.as_str().to_owned();
        let mut stores = self.stores.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(store) = stores.get(&key) {
            return Ok(Arc::clone(store));
        }
        let opened = Arc::new(
            RedbPointStore::open(&self.store_path(collection)).map_err(|e| storage_status(&e))?,
        );
        stores.insert(key, Arc::clone(&opened));
        Ok(opened)
    }
}
