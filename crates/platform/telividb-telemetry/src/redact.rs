//! What must never reach a telemetry pipeline, and how to emit the rest.
//!
//! **Telemetry bypasses every control in the security model.** Logs and traces
//! routinely land in systems with weaker access control than the database, are
//! retained longer, and are read by people who were never granted anything.
//! A pipeline that records query vectors hands out exactly what separating
//! `search` from `read_vector` was designed to prevent — and a query vector can
//! be inverted back toward its source text.
//!
//! The rules, in order of how badly they bite:
//!
//! 1. **Never emit a query or stored vector.** Not truncated, not the first few
//!    dimensions, not a "sample". Emit the dimension count if you need shape.
//! 2. **Never emit payload contents or source text.**
//! 3. **Never emit a vault's resource name.** That `vault/therapy-notes` exists
//!    is itself the disclosure.
//! 4. **Resource names are hashed by default**, so traces can still be
//!    correlated without carrying the identity around.

use std::hash::{DefaultHasher, Hash, Hasher};

/// A resource name reduced to a stable, non-reversible token.
///
/// Correlation across spans survives; the name does not. Use wherever a
/// resource identifies user content rather than schema.
///
/// Not a cryptographic hash — it resists casual reading, not a determined
/// attacker with a candidate list. For anything stronger, do not emit at all.
pub fn resource_token(name: &str) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    format!("r_{:016x}", hasher.finish())
}

/// Describe a vector without disclosing it.
///
/// The shape is almost always what you actually wanted in a span; the values
/// never are.
pub fn vector_shape(vector: &[f32]) -> VectorShape {
    VectorShape { dim: vector.len() }
}

/// Emittable facts about a vector.
///
/// `Serialize` so it can go straight onto a log record's structured data as
/// `{"dim": 768}` rather than being flattened to a string — a collector can
/// then filter on the dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct VectorShape {
    /// Number of components. Discloses nothing about their values.
    pub dim: usize,
}

impl std::fmt::Display for VectorShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dim={}", self.dim)
    }
}

/// Whether a collection name refers to a vault, and so must not be emitted
/// even as a metric label.
///
/// Vault collections are excluded from telemetry entirely rather than hashed:
/// a stable token still reveals *that* a distinct private collection is being
/// queried, and how often.
pub fn is_vault(collection: &str) -> bool {
    collection.starts_with("vault/") || collection.starts_with("vaults/")
}

/// The value to emit for a collection — the name, or a fixed placeholder when
/// it is a vault.
pub fn collection_label(collection: &str) -> &str {
    if is_vault(collection) {
        "<vault>"
    } else {
        collection
    }
}

#[cfg(test)]
#[path = "redact_test.rs"]
mod tests;
