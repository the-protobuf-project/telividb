//! The curated set, and looking things up in it.

use super::CatalogEntry;
use crate::{Error, Result};
use telividb_core::Modality;

/// Every model this build offers.
///
/// Curated rather than searched, and the reason is checkable rather than
/// editorial: the encoder loads exactly the architectures in
/// [`Architecture`](telividb_core::Architecture). Model hosts carry tens of
/// thousands of GGUF files and nearly all are generative models this loader
/// cannot read — so a search box over one would mostly return files that fail.
/// The host is where the bytes live; the curation is what makes them usable.
#[derive(Debug, Clone)]
pub struct Catalog {
    /// Entries in the order the manifest lists them, which is the order shown.
    entries: Vec<CatalogEntry>,
}

/// The manifest, compiled in.
///
/// Compiled rather than fetched so the catalog works with no network and
/// cannot be changed under a running install. Adding a model is a commit,
/// reviewed like any other change.
const BUILTIN: &str = include_str!("../../catalog/models.toml");

impl Catalog {
    /// The catalog this build ships.
    ///
    /// Panics if the compiled-in manifest is malformed, which is a build
    /// defect rather than a runtime condition — and a test parses it, so the
    /// panic is unreachable from a shipped binary.
    pub fn builtin() -> Self {
        Self::parse(BUILTIN).expect("the compiled-in catalog must parse")
    }

    /// Read a catalog from TOML.
    pub fn parse(source: &str) -> Result<Self> {
        /// The manifest's shape: a list of entries under one key.
        #[derive(serde::Deserialize)]
        struct Manifest {
            /// One per `[[model]]` block.
            model: Vec<CatalogEntry>,
        }

        let manifest: Manifest =
            toml::from_str(source).map_err(|e| Error::Catalog(e.to_string()))?;

        // The architecture gate runs during deserialization; this is the other
        // half of the same promise. An entry whose modality has no encoder
        // behind it would be offered, downloaded, and then found unusable — so
        // it is refused where the catalog is built rather than where it is
        // shown. Enforced rather than merely tested, because a manifest is data
        // and the test only covers the one committed here.
        if let Some(bad) = manifest.model.iter().find(|e| !e.is_usable()) {
            return Err(Error::Catalog(format!(
                "{}: {} models cannot be run by this engine, so the catalog \
                 must not offer one",
                bad.id, bad.modality
            )));
        }

        Ok(Self {
            entries: manifest.model,
        })
    }

    /// Every entry, in manifest order.
    pub fn entries(&self) -> &[CatalogEntry] {
        &self.entries
    }

    /// Look one up by id.
    pub fn get(&self, id: &str) -> Option<&CatalogEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Look one up by id, or say which id was not found.
    pub fn require(&self, id: &str) -> Result<&CatalogEntry> {
        self.get(id)
            .ok_or_else(|| Error::UnknownModel(id.to_owned()))
    }

    /// The default offer for someone with no preference.
    ///
    /// Exactly one entry is marked, so this is the model a first run installs
    /// without asking — the point of the catalog is that nobody has to choose
    /// before they can use the product.
    pub fn recommended(&self) -> Option<&CatalogEntry> {
        self.entries.iter().find(|e| e.recommended)
    }

    /// Entries that embed a given kind of content.
    ///
    /// Returns nothing for every modality but text today, which is the honest
    /// answer rather than an empty category hidden from the caller — the
    /// window can then say what is missing instead of showing a blank list.
    pub fn by_modality(&self, modality: Modality) -> impl Iterator<Item = &CatalogEntry> {
        self.entries.iter().filter(move |e| e.modality == modality)
    }
}

#[cfg(test)]
#[path = "catalog_test.rs"]
mod tests;
