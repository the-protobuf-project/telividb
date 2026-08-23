//! References to source content.
//!
//! The database stores a reference, not the media. Video is GB-scale and blob
//! storage is a solved problem. The hash is what detects that a source changed
//! and its embeddings are stale. See ARCHITECTURE.md §4.3.

/// A pointer to the bytes a point was derived from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRef {
    /// Where the source lives: `file://`, `s3://`, `https://`.
    pub uri: String,
    /// Byte range within the source, when the point covers only part of it.
    pub byte_range: Option<(u64, u64)>,
    /// Content hash of the referenced bytes. Stale-detection depends on this.
    pub sha256: Option<[u8; 32]>,
    /// Source text, inlined when small enough to keep re-embedding possible.
    ///
    /// This is what makes changing the embedding model feasible later: without
    /// a reachable source, a collection is locked to the model that created it.
    pub inline: Option<String>,
}

impl ContentRef {
    /// A reference to a whole source, with no range, hash or inlined copy.
    pub fn uri(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            byte_range: None,
            sha256: None,
            inline: None,
        }
    }

    /// Attach an inlined copy of the source text.
    ///
    /// Only for sources small enough to store; this is what keeps a point
    /// re-embeddable without fetching the original.
    pub fn with_inline(mut self, text: impl Into<String>) -> Self {
        self.inline = Some(text.into());
        self
    }

    /// Whether this point can be re-embedded without fetching the source.
    pub fn is_self_contained(&self) -> bool {
        self.inline.is_some()
    }
}
