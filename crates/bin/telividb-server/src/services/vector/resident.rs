//! Which model is loaded, and swapping it without a restart.
//!
//! Split from `embed.rs` because that file is about *using* a model and this is
//! about *having* one. They change for different reasons: a new task type
//! touches the first, and the loading lifecycle — discovery at startup, a swap
//! after an install — touches only this.
//!
//! The swap is the point. Installing a model used to require a restart, and
//! before that it did nothing at all, because the engine only ever read
//! `TELIVIDB_MODEL`. A shared slot is what lets the models service load one
//! into the running server while the point service goes on serving.

use std::sync::Arc;
use telividb_core::Fingerprint;
use telividb_embed::{GgmlInferencer, ModelId};

/// What is currently resident, if anything.
///
/// Split from [`Embeddings`] so the pair can be replaced together. A model and
/// the identity its vectors are bound to must change in one step: a reader that
/// saw a new model against an old id would attribute vectors to weights that
/// did not produce them, which is the provenance mixing rule 12 forbids.
#[derive(Default)]
pub(super) struct Resident {
    /// `Arc` because embedding runs on the blocking pool: the handle is cloned
    /// into `spawn_blocking`, not borrowed across an await.
    pub(super) inference: Option<Arc<GgmlInferencer>>,
    /// The identity of those weights, including the digest actually read.
    pub(super) model: Option<ModelId>,
}

/// The inference server, as the point service sees it.
///
/// Optional because a deployment that only ever receives pre-computed vectors
/// needs no model, and loading one costs hundreds of megabytes of residency it
/// would never use. A text request against a server with no model is *refused*
/// rather than ignored — silently storing nothing would look like success.
///
/// # Why this is shared rather than owned
///
/// Installing a model has to take effect without a restart. Every clone of this
/// type therefore points at one slot: the models service swaps a model in, and
/// the point service — cloned per request by `tonic` long before — sees it on
/// its next call.
///
/// An `RwLock` rather than a swap primitive because the read is trivially short:
/// it clones two `Arc`s and releases. Inference itself runs outside the lock, on
/// the blocking pool, so a long embed never blocks an install and an install
/// never stalls a query already in flight.
#[derive(Clone, Default)]
pub struct Embeddings {
    /// The one slot every clone shares.
    resident: Arc<std::sync::RwLock<Resident>>,
}

impl Embeddings {
    /// Load `path` and hold it resident for the process's life.
    ///
    /// Eager, at startup, rather than on first use: rule 45 forbids a
    /// load-run-unload cycle, and a first request that silently blocks for
    /// several seconds on a model load is indistinguishable from one that has
    /// hung.
    pub fn load(path: &std::path::Path, name: &str) -> Result<Self, telividb_embed::Error> {
        let empty = Self::default();
        empty.install(path, name)?;
        Ok(empty)
    }

    /// Load `path` into this slot, replacing whatever was there.
    ///
    /// This is what makes an installed model usable without a restart. The load
    /// happens *before* the lock is taken, so a model that takes seconds to read
    /// does not block queries against the model already resident — and a load
    /// that fails leaves the previous one in place rather than a server with
    /// nothing.
    pub fn install(&self, path: &std::path::Path, name: &str) -> Result<(), telividb_embed::Error> {
        let mut inference = GgmlInferencer::new();
        let id = ModelId::new(name, Fingerprint::unset());
        inference.register(&id, path)?;

        // The digest the file actually had, not the unset one asked for —
        // that is the identity a field's vectors are bound to.
        let model = ModelId::new(name, resident_digest(&inference, &id));

        let mut slot = self.write();
        slot.inference = Some(Arc::new(inference));
        slot.model = Some(model);
        Ok(())
    }

    /// Whether a model is resident.
    pub fn is_enabled(&self) -> bool {
        self.read().inference.is_some()
    }

    /// The name of the resident model, if there is one.
    pub fn model_name(&self) -> Option<String> {
        self.read().model.as_ref().map(|m| m.name.clone())
    }

    /// Read the slot, treating a poisoned lock as empty.
    ///
    /// A panic while holding the write lock means a load failed in a way this
    /// process cannot reason about; reporting "no model" is the fail-closed
    /// answer, and it refuses text rather than embedding with unknown weights.
    pub(super) fn read(&self) -> std::sync::RwLockReadGuard<'_, Resident> {
        self.resident.read().unwrap_or_else(|e| e.into_inner())
    }

    /// Take the slot for replacement.
    fn write(&self) -> std::sync::RwLockWriteGuard<'_, Resident> {
        self.resident.write().unwrap_or_else(|e| e.into_inner())
    }
}

/// The digest the inference server actually loaded.
fn resident_digest(inference: &GgmlInferencer, id: &ModelId) -> Fingerprint {
    inference
        .resident_digest(&id.name)
        .unwrap_or_else(Fingerprint::unset)
}
