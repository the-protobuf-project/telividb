//! The per-collection vector state `PointsSvc` holds between requests.
//!
//! Composed here rather than in `telividb-storage` because searching needs
//! both halves: storage supplies the [`VectorStore`]s, `telividb-index`
//! supplies the algorithm, and layering forbids storage from naming the index
//! (invariant 6). The composition root is the one place allowed to know both.
//!
//! **Why this is stateful when the rest of the service is not.** Every other
//! RPC opens a store, answers, and drops it. A vector field cannot work that
//! way: its unsealed buffer *is* the newest data, and dropping it between
//! requests would discard every write since the last seal. So the fields live
//! here, keyed by collection, for the life of the process.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use telividb_core::{Dim, Fingerprint, Metric, ResourceName, Result};
use telividb_index::adapters::GpuFlatIndex;
use telividb_storage::{DEFAULT_SEAL_BYTES, VectorField};

/// Everything one collection holds for one named vector field.
pub(super) struct FieldState {
    pub(super) field: VectorField,
    /// Rebuilt lazily after a write, because a GPU upload is far too expensive
    /// to redo per query — and pointless while nothing has changed.
    pub(super) index: Option<GpuFlatIndex>,
}

/// Vector fields, keyed by `(collection, field)`.
///
/// A `Mutex` rather than anything finer-grained: appends mutate the WAL and
/// buffer, so they serialise regardless, and search is not yet the hot path
/// this would need to be optimised for. Benchmark before changing it.
pub struct VectorFields {
    data_dir: PathBuf,
    fields: Mutex<HashMap<(String, String), FieldState>>,
}

impl VectorFields {
    /// Serve vector fields from underneath `data_dir`.
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            fields: Mutex::new(HashMap::new()),
        }
    }

    /// Append `vector` to `field` of `collection`, returning its row.
    ///
    /// Opens the field on first use, recovering whatever the WAL still holds.
    /// The width of the first vector written defines the field's dimension —
    /// an interim stand-in for the schema, which is where a declared `dim`
    /// will come from once `CreateCollection` resolves a descriptor set.
    pub fn append(&self, collection: &ResourceName, field: &str, vector: &[f32]) -> Result<usize> {
        let mut fields = self.lock();
        let key = (collection.as_str().to_owned(), field.to_owned());

        let dim = Dim::new(vector.len() as u32)?;
        let state = match fields.entry(key) {
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(self.open_field(collection, field, dim)?)
            }
        };

        let row = state.field.append(vector).map_err(storage_err)?;
        state.field.commit().map_err(storage_err)?;
        state
            .field
            .seal_if_needed(DEFAULT_SEAL_BYTES)
            .map_err(storage_err)?;

        // The corpus changed, so any index built from it is stale. Rebuilt on
        // the next search rather than here: a write that also paid for a GPU
        // upload would make ingest as slow as the thing it feeds.
        state.index = None;
        Ok(row)
    }

    /// Open one field from disk, recovering whatever its WAL still holds.
    ///
    /// `dim` comes from the vector at hand — the width of the first vector
    /// written, or of the query. An interim stand-in for the schema, which is
    /// where a declared dimension will come from once `CreateCollection`
    /// resolves a descriptor set.
    pub(super) fn open_field(
        &self,
        collection: &ResourceName,
        field: &str,
        dim: Dim,
    ) -> Result<FieldState> {
        let opened = VectorField::open(
            self.dir_for(collection, field),
            field,
            dim,
            Metric::Cosine,
            schema_fingerprint(),
            model_fingerprint(),
        )
        .map_err(storage_err)?;
        Ok(FieldState {
            field: opened,
            index: None,
        })
    }

    /// Where one field's WAL, segments and manifest live.
    pub(super) fn dir_for(&self, collection: &ResourceName, field: &str) -> PathBuf {
        self.data_dir
            .join(collection.leaf())
            .join("vectors")
            .join(field)
    }

    pub(super) fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<(String, String), FieldState>> {
        // A poisoned lock means a previous request panicked mid-append. The
        // state is still structurally valid — the WAL is the authority — so
        // recovering is better than making every later request fail too.
        self.fields.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Interim schema identity.
///
/// Every field is written under one fingerprint because no descriptor set is
/// resolved yet — `CreateCollection` is still `unimplemented`. When it lands,
/// this becomes the collection's real schema fingerprint and segments written
/// now will not validate against it, which is the correct behaviour: they were
/// written under a schema that no longer exists.
fn schema_fingerprint() -> Fingerprint {
    Fingerprint::of(b"telividb.interim.schema.v1")
}

/// Interim model identity.
///
/// Rule 12 binds a field to one embedding model, checked by this fingerprint.
/// Until the inference server exists there is no model to name, so every field
/// shares a placeholder — which means the provenance check currently passes
/// trivially rather than being enforced.
fn model_fingerprint() -> Fingerprint {
    Fingerprint::of(b"telividb.interim.model.v1")
}

/// Fold a storage failure into the domain error type.
///
/// `PointStore` rather than `GpuIndex`: a WAL or segment failure has nothing to
/// do with the device, and mislabelling it sends whoever reads the log to the
/// wrong subsystem.
fn storage_err(e: telividb_storage::Error) -> telividb_core::Error {
    telividb_core::Error::PointStore {
        reason: e.to_string(),
    }
}
