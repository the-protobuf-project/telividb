//! The registry of resident models, and the [`Inferencer`] built over it.

use super::resident::ResidentModel;
use crate::domain::{ModelId, Pooling};
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::Path;
use telividb_compute::Backend;

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
pub struct GgmlInferencer {
    pub(super) models: HashMap<String, ResidentModel>,
    /// Cap on sequence length, at or below each model's own context.
    ///
    /// `None` uses whatever the model covers.
    pub(super) max_tokens: Option<usize>,
}

impl GgmlInferencer {
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
        let backend = Backend::best().map_err(|e| crate::error::Error::Compute(e.to_string()))?;
        let model = ResidentModel::load(id, path, backend, pooling)?;
        self.models.insert(id.name.clone(), model);
        Ok(self)
    }

    /// Truncate every sequence at `max_tokens`, below the model's context.
    ///
    /// **This trades accuracy for speed, and the trade is usually good.**
    /// Attention is quadratic in sequence length, so halving the cap quarters
    /// the attention work — while the tail being dropped is, for most corpora,
    /// the part of a document that contributed least. BEIR's own evaluations
    /// cap BERT-family models at 512 for exactly this reason.
    ///
    /// It is *not* free: a corpus of genuinely long documents whose meaning
    /// lives past the cap will lose recall, silently. Measure before setting
    /// it, and prefer the model's full context when documents are short enough
    /// that the cap never binds.
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
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

    /// The digest a resident model actually loaded under.
    ///
    /// A caller that registered with [`Fingerprint::unset`] — "whatever this
    /// file is" — needs this to learn the identity it got, because that digest
    /// is what a vector field's provenance is bound to (rule 12). Without it,
    /// the binding could only ever be recorded as "unset", which records
    /// nothing.
    ///
    /// [`Fingerprint::unset`]: telividb_core::Fingerprint::unset
    pub fn resident_digest(&self, name: &str) -> Option<telividb_core::Fingerprint> {
        self.models.get(name).map(|m| m.id.fingerprint)
    }

    /// Look up a model, verifying the caller's digest against what is loaded.
    pub(super) fn resolve(&self, id: &ModelId) -> Result<&ResidentModel> {
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

#[cfg(test)]
#[path = "inferencer_test.rs"]
mod tests;
