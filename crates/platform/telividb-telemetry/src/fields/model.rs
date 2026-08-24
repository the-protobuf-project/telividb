//! Field names describing a resident model and what it was asked to do.
//!
//! Split from the parent purely for file length (rule 13); every name here is
//! re-exported by [`super`], so a call site never spells this module.

/// Model name, e.g. `nomic-embed-text-v1.5`. **Label-safe** — a deployment
/// registers a fixed, small set of models up front (rule 45), so the value
/// space is bounded by configuration the way [`FIELD`]'s is by schema.
pub const MODEL: &str = "telividb.model";

/// Short digest of the GGUF a model was loaded from. Span-only: it is
/// high-cardinality across deployments and is what tells two builds of the
/// same model apart when recall changes and nothing else did.
pub const MODEL_FINGERPRINT: &str = "telividb.model.fingerprint";

/// How token states collapse to one vector: `mean` or `cls`. **Label-safe.**
pub const POOLING: &str = "telividb.model.pooling";

/// Which side of a search text was embedded for: `document` or `query`.
/// **Label-safe** — a closed set of two.
///
/// Worth separating: the two take different task prefixes, and a deployment
/// embedding everything as one of them is a recall bug that shows up here
/// before it shows up in a recall number.
pub const TASK: &str = "telividb.model.task";
