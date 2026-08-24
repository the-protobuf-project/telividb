//! One named vector field, as declared in the schema.

use crate::{Dim, Fingerprint, Metric};

/// Which search algorithm a field is indexed with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    /// Exhaustive. Ground truth, and correct for small fields.
    Flat,
    /// Hierarchical navigable small world graph. The default above trivial sizes.
    Hnsw,
    /// Inverted file with product quantization, for memory-constrained fields.
    IvfPq,
}

impl IndexKind {
    /// The name used in configuration and telemetry.
    pub fn as_str(self) -> &'static str {
        match self {
            IndexKind::Flat => "flat",
            IndexKind::Hnsw => "hnsw",
            IndexKind::IvfPq => "ivfpq",
        }
    }

    /// Whether results from this index are exhaustive.
    pub fn is_exact(self) -> bool {
        matches!(self, IndexKind::Flat)
    }
}

/// Everything the index and the embedder need about one vector field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorFieldSpec {
    /// Field name, e.g. `image_clip`. Unique within a point type.
    /// Field name, e.g. `image_clip`. Unique within a point type.
    pub name: String,
    /// Vector width. Every row in this field is exactly this wide.
    pub dim: Dim,
    /// How similarity is measured.
    pub metric: Metric,
    /// Which search algorithm indexes this field.
    pub index: IndexKind,

    /// The model that produces vectors for this field.
    pub model: String,
    /// Digest of that model file, mirrored into the field header on write.
    pub model_fingerprint: Fingerprint,

    /// The encoder that handles *queries* against this field.
    ///
    /// For a joint model this is the other tower: searching an image field with
    /// text must encode that text with the model's text tower, not with the
    /// collection's text embedder. Declared here rather than left to runtime
    /// convention because getting it wrong yields plausible garbage rather than
    /// an error — the worst failure available.
    pub query_encoder: Option<String>,

    /// Permission gating access to raw vectors of this field.
    ///
    /// Per-field rather than per-collection: allowing search over transcripts
    /// while denying voiceprints is a routine requirement, and a voiceprint is
    /// biometric data where a transcript is not.
    pub permission: Option<String>,
}

impl VectorFieldSpec {
    /// Minimal spec, for tests and for fields declared without provenance.
    /// A field with no declared model, index or permission.
    ///
    /// For tests and for fields whose provenance is not yet bound.
    pub fn new(name: impl Into<String>, dim: Dim, metric: Metric) -> Self {
        Self {
            name: name.into(),
            dim,
            metric,
            index: IndexKind::Flat,
            model: String::new(),
            model_fingerprint: Fingerprint::unset(),
            query_encoder: None,
            permission: None,
        }
    }

    /// Whether a query against this field must be encoded by a different model
    /// than the one that produced its stored vectors.
    pub fn needs_cross_encoder(&self) -> bool {
        self.query_encoder
            .as_ref()
            .is_some_and(|q| !q.is_empty() && *q != self.model)
    }
}

#[cfg(test)]
#[path = "vector_field_test.rs"]
mod tests;
