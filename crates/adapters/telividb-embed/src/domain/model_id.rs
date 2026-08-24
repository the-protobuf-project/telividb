//! Which model, exactly.

use telividb_core::Fingerprint;

/// A model's identity: a name to call it by, and the digest that says what it
/// actually is.
///
/// The digest is the identity and the name is a label. Rule 12 binds a vector
/// field to a model *identity* because vectors from two different models
/// merged into one index do not fail — recall degrades and every neighbour
/// returned stays plausible. A name alone cannot carry that guarantee: the
/// same name is routinely reused across quantizations and fine-tunes, which
/// are exactly the cases that must not be mixed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelId {
    /// Human-facing label, as written in configuration and telemetry.
    pub name: String,
    /// SHA-256 of the GGUF file, which is what actually identifies the weights.
    pub fingerprint: Fingerprint,
}

impl ModelId {
    /// Name a model by its label and its file digest.
    pub fn new(name: impl Into<String>, fingerprint: Fingerprint) -> Self {
        Self {
            name: name.into(),
            fingerprint,
        }
    }
}

impl std::fmt::Display for ModelId {
    /// Label plus a short digest prefix — enough to tell two builds of the
    /// same model apart in a log line without printing 64 hex characters.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.fingerprint.short())
    }
}
