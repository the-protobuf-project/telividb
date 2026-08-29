//! One curated model.

use telividb_core::{Architecture, Fingerprint, Modality};

/// A model the catalog offers, verified against the file it names.
///
/// Every field below was read from the actual GGUF or from the host's API at
/// curation time, never copied from a model card. That matters most for
/// [`architecture`](Self::architecture) and [`digest`](Self::digest): the first
/// decides whether the file can be loaded at all, and the second is the only
/// thing standing between a download and whatever a host happens to serve.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CatalogEntry {
    /// Stable identifier, used in configuration and as the installed filename.
    ///
    /// Never reused for different weights: it names a specific file, and two
    /// files under one id is exactly the provenance mixing rule 12 forbids.
    pub id: String,
    /// Name shown in the window.
    pub display_name: String,
    /// What this model is good for, in a sentence a non-specialist can act on.
    ///
    /// Written to help someone choose — bigger is not simply better, and the
    /// trade is index size and query latency against recall.
    pub description: String,
    /// The host repository, as `owner/name`.
    pub repository: String,
    /// The GGUF file within that repository.
    pub file: String,
    /// SHA-256 of the file.
    ///
    /// The identity, not the label (rule 12). Checked after download; a
    /// mismatch is refused rather than loaded, because a file that is not the
    /// one curated has none of the properties recorded here.
    #[serde(deserialize_with = "super::entry_serde::digest")]
    pub digest: Fingerprint,
    /// Exact size in bytes.
    ///
    /// Exact rather than rounded, because it is what a progress indicator
    /// divides by — an approximate total produces a bar that finishes early or
    /// never arrives.
    pub size_bytes: u64,
    /// What this model embeds.
    ///
    /// Every entry is [`Modality::Text`] today. See that type for why the
    /// others are not simply a catalog entry away.
    #[serde(deserialize_with = "super::entry_serde::modality")]
    pub modality: Modality,
    /// The GGUF architecture, read from the file's own header.
    ///
    /// Typed rather than a string, so an entry naming something the encoder
    /// cannot read fails when the catalog is parsed — which a test reaches —
    /// rather than when a user waits for a download and then cannot use it.
    #[serde(deserialize_with = "super::entry_serde::architecture")]
    pub architecture: Architecture,
    /// Components in the vectors it produces.
    ///
    /// A collection's vector field must declare the same width, so this is
    /// what the window offers when a field is bound to this model.
    pub dimensions: u32,
    /// Maximum input length in tokens; longer input is truncated.
    pub context_length: u32,
    /// The quantization the file carries, such as `Q8_0`.
    pub quantization: String,
    /// SPDX identifier for the weights' licence.
    pub license: String,
    /// Whether this is the default offer for someone with no preference.
    ///
    /// Exactly one entry sets this, and a test enforces that — two defaults is
    /// a choice presented as a recommendation, which helps nobody.
    pub recommended: bool,
}

impl CatalogEntry {
    /// Where the file is downloaded from.
    ///
    /// Built from the repository and filename rather than stored, so a catalog
    /// entry cannot name one repository and quietly fetch from another.
    pub fn download_url(&self) -> String {
        format!(
            "https://huggingface.co/{}/resolve/main/{}?download=true",
            self.repository, self.file
        )
    }

    /// The human-facing page for these weights.
    ///
    /// Offered beside every entry so a choice can be checked — licence,
    /// provenance, benchmarks — against the publisher rather than against this
    /// catalog's one-line summary.
    pub fn repository_url(&self) -> String {
        format!("https://huggingface.co/{}", self.repository)
    }

    /// Whether this engine can load and run this entry.
    ///
    /// Always true for a catalog entry, because parsing rejects anything else.
    /// It exists for entries built from a *user-supplied* repository or URL,
    /// which travel as the same type and have no such guarantee.
    pub fn is_usable(&self) -> bool {
        self.modality.is_supported()
    }
}
