//! The registry of resident models, and the [`Inferencer`] built over it.

use super::batch;
use super::model::ResidentModel;
use crate::domain::{ModelId, Pooling, Task};
use crate::error::{Error, Result};
use crate::ports::Inferencer;
use candle_core::Tensor;
use std::collections::HashMap;
use std::path::Path;
use telividb_core::Dim;

/// Holds every model the process can run, all of them loaded.
///
/// Ollama-shaped and deliberately so: models are registered up front by the
/// composition root and then simply *are* resident. There is no load-on-demand
/// path, because rule 45 forbids the load-run-unload cycle that would defeat
/// batching — and because a first request that silently blocks for ten seconds
/// on a model load is indistinguishable from one that has hung.
///
/// Keyed by name rather than by digest so a caller can ask for `nomic-embed`
/// without knowing which build is deployed; the digest is then *verified*
/// against what is resident, so the convenience never becomes a way to get
/// different weights than the ones a field is bound to.
#[derive(Default)]
pub struct CandleInferencer {
    models: HashMap<String, ResidentModel>,
}

impl CandleInferencer {
    /// An inference server with nothing resident yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load `path` and keep it resident under `id`, taking the pooling mode
    /// the model itself declares.
    ///
    /// Takes `&mut self` — registration is the composition root's job and
    /// happens before serving, while [`Inferencer::embed`] takes `&self` so
    /// concurrent callers can be batched together.
    pub fn register(&mut self, id: &ModelId, path: &Path) -> Result<&mut Self> {
        self.register_with_pooling(id, path, None)
    }

    /// The same, overriding the declared pooling mode.
    ///
    /// Separate rather than an argument on [`Self::register`] because the
    /// override is the rare case: a model that declares its pooling should be
    /// believed, and a caller passing one on every call is a caller who will
    /// eventually pass the wrong one.
    pub fn register_with_pooling(
        &mut self,
        id: &ModelId,
        path: &Path,
        pooling: Option<Pooling>,
    ) -> Result<&mut Self> {
        let model = ResidentModel::load(id, path, pooling)?;
        self.models.insert(id.name.clone(), model);
        Ok(self)
    }

    /// How many models are resident.
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// Whether nothing is resident yet.
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    /// The names of every resident model, sorted — the `ollama ps` view.
    pub fn resident_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.models.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Look up a model, verifying the caller's digest against what is loaded.
    fn resolve(&self, id: &ModelId) -> Result<&ResidentModel> {
        let model = self
            .models
            .get(&id.name)
            .ok_or_else(|| Error::NotResident(id.name.clone()))?;

        // An unset digest means the caller did not pin one, which is allowed
        // for ad-hoc use. A digest that is set and *differs* is refused: it
        // means the field was bound to other weights, and mixing provenance
        // inside one index degrades recall with nothing reporting it.
        if !id.fingerprint.is_unset() && id.fingerprint != model.id.fingerprint {
            return Err(Error::DigestMismatch {
                name: id.name.clone(),
                expected: id.fingerprint.short(),
                found: model.id.fingerprint.short(),
            });
        }
        Ok(model)
    }
}

impl Inferencer for CandleInferencer {
    fn embed(&self, model: &ModelId, task: Task, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let model = self.resolve(model)?;
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let encoder = model.encoder();
        let batch = batch::encode(
            model.tokenizer(),
            texts,
            task,
            encoder.context(),
            encoder.device(),
        )?;
        let pooled = encoder.forward(&batch.ids, &batch.attention, model.pooling)?;
        Ok(normalize(&pooled)?.to_vec2()?)
    }

    fn dim(&self, model: &ModelId) -> Result<Dim> {
        self.resolve(model)?.dim()
    }

    fn is_resident(&self, model: &ModelId) -> bool {
        self.resolve(model).is_ok()
    }
}

/// Scale each row to unit length.
///
/// Done here, once, rather than left to the caller. The storage layer's cosine
/// path is dot-product over pre-normalized vectors (see CLAUDE.md's cosine
/// note), so an un-normalized vector reaching it does not error — it ranks by
/// magnitude as much as by direction, which looks like a quality problem
/// rather than a bug.
///
/// The clamp guards a genuinely zero row, which a text that tokenizes to
/// nothing can produce: dividing by its zero norm yields `NaN`, and a `NaN`
/// vector silently poisons every comparison it takes part in.
fn normalize(xs: &Tensor) -> candle_core::Result<Tensor> {
    let norm = xs
        .sqr()?
        .sum_keepdim(candle_core::D::Minus1)?
        .sqrt()?
        .clamp(1e-12, f64::INFINITY)?;
    xs.broadcast_div(&norm)
}

#[cfg(test)]
#[path = "inferencer_test.rs"]
mod tests;
