//! Turning text into vectors, server-side.
//!
//! **Why the server and not the client.** One inference server serves ingest,
//! query encoding and every plugin's compute step alike (CLAUDE.md rules
//! 42–45). That is what makes the model a property of the *field* rather than
//! of whichever client happened to write to it: a client with its own model
//! would be a second path to the same index, and vectors from two models
//! merged into one field degrade recall with nothing anywhere reporting it
//! (rule 12).
//!
//! It is also the only arrangement in which the policy check at this boundary
//! (rule 44) can mean anything — a check a caller can route around is not a
//! check. That check is not wired yet; this is the boundary it will attach to.

use std::sync::Arc;
use telividb_buffers::protobuf::point::v1::{NamedVector, Point as WirePoint};
use telividb_core::Fingerprint;
use telividb_embed::{GgmlInferencer, Inferencer, ModelId, Task};
use telividb_telemetry::{fields, logger};
use tonic::Status;

/// What is currently resident, if anything.
///
/// Split from [`Embeddings`] so the pair can be replaced together. A model and
/// the identity its vectors are bound to must change in one step: a reader that
/// saw a new model against an old id would attribute vectors to weights that
/// did not produce them, which is the provenance mixing rule 12 forbids.
#[derive(Default)]
struct Resident {
    /// `Arc` because embedding runs on the blocking pool: the handle is cloned
    /// into `spawn_blocking`, not borrowed across an await.
    inference: Option<Arc<GgmlInferencer>>,
    /// The identity of those weights, including the digest actually read.
    model: Option<ModelId>,
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
    fn read(&self) -> std::sync::RwLockReadGuard<'_, Resident> {
        self.resident.read().unwrap_or_else(|e| e.into_inner())
    }

    /// Take the slot for replacement.
    fn write(&self) -> std::sync::RwLockWriteGuard<'_, Resident> {
        self.resident.write().unwrap_or_else(|e| e.into_inner())
    }

    /// Replace any `text` in `point`'s named vectors with a computed vector.
    ///
    /// Batched: every field's text is embedded in one call, because padding a
    /// batch to its longest sequence is what lets the device do useful work.
    pub async fn resolve_point(&self, point: &mut WirePoint) -> Result<(), Status> {
        let pending: Vec<(usize, String)> = point
            .vectors
            .iter()
            .enumerate()
            .filter(|(_, named)| !named.text.is_empty())
            .map(|(i, named)| (i, named.text.clone()))
            .collect();

        for named in &point.vectors {
            check_exactly_one(named)?;
        }
        if pending.is_empty() {
            return Ok(());
        }

        let texts: Vec<String> = pending.iter().map(|(_, t)| t.clone()).collect();
        let vectors = self.embed(Task::Document, texts).await?;

        for ((index, _), vector) in pending.into_iter().zip(vectors) {
            point.vectors[index].vector = Some(super::convert::vector_to_wire(&vector));
            // Cleared so the stored point does not also claim to be unresolved
            // text, which a later reader would try to embed again.
            point.vectors[index].text = String::new();
        }
        Ok(())
    }

    /// Embed one query.
    pub async fn embed_query(&self, text: &str) -> Result<Vec<f32>, Status> {
        let mut vectors = self.embed(Task::Query, vec![text.to_owned()]).await?;
        if vectors.is_empty() {
            return Err(Status::internal("the inference server returned no vector"));
        }
        Ok(vectors.remove(0))
    }

    /// Run the model on the blocking pool.
    ///
    /// Inference is compute-bound and synchronous — a GPU dispatch and a wait.
    /// Running it on a tonic executor thread would stall every other request
    /// sharing that thread, which invariant 5 forbids.
    async fn embed(&self, task: Task, texts: Vec<String>) -> Result<Vec<Vec<f32>>, Status> {
        // Cloned out of the slot and released immediately: inference runs
        // outside the lock, so an install can proceed while this is computing.
        let (inference, model) = {
            let slot = self.read();
            match (&slot.inference, &slot.model) {
                (Some(i), Some(m)) => (Arc::clone(i), m.clone()),
                _ => {
                    return Err(Status::failed_precondition(
                        "this server has no embedding model loaded, so it cannot \
                         accept text. Install one from the model catalog, or \
                         start the server with --model <path-to.gguf>",
                    ));
                }
            }
        };

        logger::debug!("embedding text").with_data(&serde_json::json!({
            fields::MODEL: model.name,
            fields::TASK: task.as_str(),
            fields::RECORDS: texts.len(),
        }));

        tokio::task::spawn_blocking(move || inference.embed(&model, task, &texts))
            .await
            .map_err(|e| Status::internal(format!("embedding task failed: {e}")))?
            .map_err(|e| Status::internal(format!("embedding failed: {e}")))
    }
}

/// Refuse a named vector that carries both a vector and text, or neither.
///
/// Both is ambiguous — there is no reason to prefer one, and picking silently
/// would store something the caller did not ask for. Neither is simply empty.
fn check_exactly_one(named: &NamedVector) -> Result<(), Status> {
    match (named.vector.is_some(), named.text.is_empty()) {
        (true, false) => Err(Status::invalid_argument(format!(
            "field {:?} carries both a vector and text; set exactly one, \
             since there is no correct way to choose between them",
            named.field_id
        ))),
        (false, true) => Err(Status::invalid_argument(format!(
            "field {:?} carries neither a vector nor text",
            named.field_id
        ))),
        _ => Ok(()),
    }
}

/// The digest the inference server actually loaded.
fn resident_digest(inference: &GgmlInferencer, id: &ModelId) -> Fingerprint {
    inference
        .resident_digest(&id.name)
        .unwrap_or_else(Fingerprint::unset)
}
