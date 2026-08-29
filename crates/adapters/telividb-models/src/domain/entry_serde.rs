//! Turning catalog strings into the typed values the rest of the code uses.
//!
//! Separate from [`CatalogEntry`](super::CatalogEntry) because these are
//! mechanism: three near-identical adapters between `serde` and the ontology,
//! which would otherwise crowd out the entry's own documentation.

use serde::{Deserialize, Deserializer};
use telividb_core::{Architecture, Fingerprint, Modality};

/// Parse a 64-character hex digest.
pub(super) fn digest<'de, D: Deserializer<'de>>(d: D) -> Result<Fingerprint, D::Error> {
    let raw = String::deserialize(d)?;
    Fingerprint::from_hex(&raw).ok_or_else(|| {
        serde::de::Error::custom(format!(
            "{raw:?} is not a 64-character hex SHA-256; a digest is copied from \
             the host's API, never typed by hand"
        ))
    })
}

/// Parse a modality name.
pub(super) fn modality<'de, D: Deserializer<'de>>(d: D) -> Result<Modality, D::Error> {
    let raw = String::deserialize(d)?;
    Modality::parse(&raw)
        .ok_or_else(|| serde::de::Error::custom(format!("{raw:?} is not a known modality")))
}

/// Parse an architecture, refusing any the encoder cannot read.
///
/// This is the gate that keeps the catalog honest. A GGUF naming
/// `gemma-embedding` or `qwen3` is a real embedding model that this loader has
/// no forward pass for, and listing one would offer a download that cannot be
/// used after it lands.
pub(super) fn architecture<'de, D: Deserializer<'de>>(d: D) -> Result<Architecture, D::Error> {
    let raw = String::deserialize(d)?;
    Architecture::from_gguf(&raw).ok_or_else(|| {
        serde::de::Error::custom(format!(
            "{raw:?} is not an architecture this engine can load; supported: {}",
            Architecture::NAMES.join(", ")
        ))
    })
}
