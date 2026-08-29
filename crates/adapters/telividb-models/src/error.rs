//! What can go wrong acquiring a model.

use telividb_core::Fingerprint;

/// A model could not be listed, fetched or verified.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The catalog file itself is malformed.
    ///
    /// Only reachable for a catalog supplied at runtime: the built-in one is
    /// compiled in and parsed by a test, so a bad entry fails the build.
    #[error("the model catalog could not be read: {0}")]
    Catalog(String),

    /// No catalog entry has that id.
    #[error("no model called {0:?} is in the catalog")]
    UnknownModel(
        /// The id that was asked for.
        String,
    ),

    /// The file names an architecture the encoder has no forward pass for.
    ///
    /// Refused *before* downloading rather than after. The alternative is
    /// fetching several hundred megabytes and then reporting that it cannot be
    /// used, which is the same outcome an hour later.
    #[error(
        "{name} is a {found:?} model, which this engine cannot load; it reads {supported}. \
         Adding an architecture is work in the encoder, not a catalog entry."
    )]
    UnsupportedArchitecture {
        /// What was being fetched, for a message that names something real.
        name: String,
        /// The architecture the file declares.
        found: String,
        /// The architectures that do load, comma-separated.
        supported: String,
    },

    /// The bytes that arrived are not the bytes that were curated.
    ///
    /// Fatal, never a warning. A file that fails this is not a corrupted copy
    /// of the right model — it is a different file, and every property the
    /// catalog records about it (width, context, quality) is now unverified.
    #[error(
        "{name}: expected sha256 {expected}, got {found}; the file was not what the catalog names"
    )]
    DigestMismatch {
        /// The model that was being installed.
        name: String,
        /// The digest the catalog records.
        expected: Fingerprint,
        /// The digest of what actually arrived.
        found: Fingerprint,
    },

    /// A GGUF header could not be read.
    #[error("could not read the GGUF header: {0}")]
    Gguf(String),

    /// The host could not be reached, or answered with something unusable.
    #[error("fetching {url}: {reason}")]
    Fetch {
        /// What was being fetched.
        url: String,
        /// Why it failed.
        reason: String,
    },

    /// Reading or writing the model directory failed.
    #[error("model storage: {0}")]
    Io(#[from] std::io::Error),
}

/// The result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
