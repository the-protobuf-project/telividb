//! The one compute boundary.

use crate::domain::{ModelId, Task};
use crate::error::Result;
use telividb_core::Dim;

/// Computes embeddings for text, against a model held resident.
///
/// Named `Inferencer` rather than `Embedder` because it is not the embedding
/// crate's private trait: ingest, query-time encoding and every plugin's
/// compute step come through here (rules 42–45). That is what makes the
/// policy check at this boundary meaningful — a single choke point can be
/// checked, a set of scattered model loads cannot.
///
/// `&self` rather than `&mut self`: an implementation serves concurrent
/// callers, and batching across them (the reason to have a server at all) is
/// impossible if every caller needs exclusive access.
pub trait Inferencer: Send + Sync {
    /// Embed `texts` with `model`, as documents or as queries.
    ///
    /// Batched by design — one call with many texts, not many calls with one.
    /// Padding a batch to its longest sequence is what lets a GPU do useful
    /// work; per-text calls leave it idle between kernel launches.
    ///
    /// Returns one vector per input, in the same order.
    fn embed(&self, model: &ModelId, task: Task, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// The width of the vectors `model` produces.
    ///
    /// Available without embedding anything so a collection can validate a
    /// field's declared dimension against the model bound to it (rule 12) at
    /// schema time, rather than discovering the mismatch on first ingest.
    fn dim(&self, model: &ModelId) -> Result<Dim>;

    /// Whether `model` is currently resident.
    ///
    /// Rule 45 forbids load-on-demand, so this is how a caller distinguishes
    /// "not configured" from "temporarily unavailable" — the latter does not
    /// exist here.
    fn is_resident(&self, model: &ModelId) -> bool;
}
