//! What crosses the bridge.
//!
//! Tauri serializes these as JSON, so every field name here is a field name in
//! the window's TypeScript. They are kept flat and small for that reason.

use serde::{Deserialize, Serialize};
use telividb_client::SearchResults;

/// One collection, as the sidebar needs it.
#[derive(Debug, Clone, Serialize)]
pub struct CollectionSummary {
    /// The id within the server, which is what every other call takes.
    pub id: String,
}

impl CollectionSummary {
    /// Wrap an id the client returned.
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

/// A query from the window.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    /// Collection to search.
    pub collection: String,

    /// Named vector field to search.
    ///
    /// Required by the server, and rightly: each field carries its own model
    /// and metric, so a query only means something against the field it was
    /// encoded for.
    pub field: String,

    /// Text for the server to encode, when the field has a model behind it.
    ///
    /// Exactly one of this and `vector` carries the query. Text needs a model
    /// resident on the server; without one the server refuses, and the window
    /// says so rather than showing an empty result.
    pub text: Option<String>,

    /// A query vector the caller already has.
    #[serde(default)]
    pub vector: Vec<f32>,

    /// How many neighbours to return.
    ///
    /// The `k` of a nearest-neighbour search, not a display limit: it decides
    /// how much work the index does.
    pub k: usize,
}

/// One matching point.
#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    /// The point's id within its collection.
    pub id: String,
    /// Similarity, on the scale of the field's own metric.
    pub score: f32,
    /// Text the point carries inline, when it carries any.
    pub text: Option<String>,
}

/// What a search answered.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    /// Matching points, nearest first.
    pub hits: Vec<SearchHit>,

    /// Whether every source answered.
    ///
    /// Carried rather than dropped, and shown rather than assumed. A caller
    /// handed only hits cannot tell "nothing matched" from "nothing you can
    /// currently see matched", and those are different answers.
    pub complete: bool,

    /// Vaults that were locked, by name. Names only, never contents.
    pub locked_vaults: Vec<String>,
}

impl From<SearchResults> for SearchResponse {
    fn from(results: SearchResults) -> Self {
        Self {
            complete: results.is_complete(),
            locked_vaults: results.locked_vaults().to_vec(),
            hits: results
                .into_hits()
                .into_iter()
                .map(|hit| SearchHit {
                    id: hit.name,
                    score: hit.score,
                    text: hit.text,
                })
                .collect(),
        }
    }
}
