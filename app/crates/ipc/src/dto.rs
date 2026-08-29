//! What crosses the bridge.
//!
//! Tauri serializes these as JSON, so every field name here is a field name in
//! the window's TypeScript. They are kept flat and small for that reason.

use serde::{Deserialize, Serialize};
use telividb_client::{Record, SearchResults};
use telividb_desktop_engine::Environment;

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

/// A request to create a collection from a shipped preset.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCollectionRequest {
    /// Which preset supplies the schema.
    pub preset: String,
    /// The collection's id, forming the final path segment of its name.
    pub collection: String,
    /// Width to declare for the text field.
    ///
    /// Must match the model that will embed into it — a field is bound to one
    /// model identity, and a mismatch is refused on the first write. `None`
    /// falls back to the BERT-family width, which is right only if that is what
    /// is loaded.
    #[serde(default)]
    pub dimensions: Option<usize>,
}

/// One row on its way in.
///
/// Text rather than a vector: a CSV has no vectors in it, and asking a person
/// to supply 768 floats per row would make import useless. The server encodes
/// through the field's own model, which is also the only way the vectors end up
/// with provenance the engine can check.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportRow {
    /// The point's id within the collection.
    pub id: String,
    /// The text to embed and store.
    pub text: String,
}

/// A batch of rows for one collection.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportRequest {
    /// Collection to write into.
    pub collection: String,
    /// Named vector field the text is encoded for.
    pub field: String,
    /// The rows, already mapped from whatever file they came out of.
    ///
    /// Empty is refused rather than treated as a no-op: a file that produced
    /// no rows is a mapping mistake, and reporting success would hide it.
    pub rows: Vec<ImportRow>,
}

/// What an import wrote.
#[derive(Debug, Clone, Serialize)]
pub struct ImportResponse {
    /// How many points were created.
    pub written: usize,
}

/// One stored point, as a table row.
#[derive(Debug, Clone, Serialize)]
pub struct PointRow {
    /// The point's id within its collection.
    pub id: String,
    /// Text the point carries inline, when it carries any.
    pub text: Option<String>,
}

impl From<Record> for PointRow {
    fn from(record: Record) -> Self {
        Self {
            id: record.name,
            text: record.text,
        }
    }
}

/// What this engine can currently do.
///
/// Small on purpose. It answers the one question the window has to ask before
/// offering an import: can the server turn text into a vector? Everything else
/// about the engine belongs to the `SystemInfo` service, which is not served
/// yet.
#[derive(Debug, Clone, Serialize)]
pub struct Capabilities {
    /// Whether an embedding model is loaded.
    ///
    /// False means text is refused — both `add_text` and a text query — and
    /// only precomputed vectors work.
    pub has_model: bool,
    /// Where the engine is listening.
    pub address: String,
    /// The compute environment, as this process found it.
    pub environment: Environment,
    /// The directory holding segments, the write-ahead log and metadata.
    pub data_dir: String,
}
