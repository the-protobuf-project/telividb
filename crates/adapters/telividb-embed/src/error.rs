//! Inference errors.

/// Failures loading a model or computing an embedding.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A filesystem operation failed.
    #[error("io: {0}")]
    Io(
        /// The underlying filesystem failure.
        #[from]
        std::io::Error,
    ),

    /// `candle` refused a tensor operation, or could not read the GGUF.
    #[error("candle: {0}")]
    Candle(
        /// The underlying tensor-runtime failure.
        #[from]
        candle_core::Error,
    ),

    /// Text could not be tokenized.
    #[error("tokenizer: {0}")]
    Tokenizer(
        /// The underlying tokenizer failure, as reported by `tokenizers`.
        String,
    ),

    /// The caller named a model that is not resident.
    ///
    /// Distinguishable from every other failure on purpose: rule 45 forbids a
    /// load-on-demand path, so "not resident" means the composition root never
    /// registered it, which is a configuration problem rather than a transient
    /// one.
    #[error("model {0} is not resident; register it before use")]
    NotResident(
        /// Name the caller asked for.
        String,
    ),

    /// The GGUF is missing something the encoder needs.
    ///
    /// Reported rather than defaulted: a hyperparameter guessed wrong produces
    /// a model that runs and returns plausible, wrong vectors — the failure
    /// mode rule 12 exists to prevent.
    #[error("gguf is missing {what}, which this encoder requires")]
    MissingFromGguf {
        /// Metadata key or tensor name that was absent.
        what: String,
    },

    /// The GGUF describes an architecture this encoder does not implement.
    #[error("unsupported architecture {found:?}; this build encodes {supported:?}")]
    UnsupportedArchitecture {
        /// `general.architecture` as recorded in the file.
        found: String,
        /// Architectures this build can run.
        supported: &'static [&'static str],
    },

    /// A model file's digest did not match the identity it was registered under.
    ///
    /// Refused rather than accepted, because a field is bound to a model
    /// *identity* (rule 12): silently accepting different weights under the
    /// same name mixes provenance inside one index, which degrades recall with
    /// nothing anywhere reporting it.
    #[error("model digest mismatch for {name}: registered {expected}, file is {found}")]
    DigestMismatch {
        /// Model name under which the mismatch was found.
        name: String,
        /// Digest the caller declared.
        expected: String,
        /// Digest the file actually has.
        found: String,
    },
}

/// Convenience alias for an inference-layer result.
pub type Result<T> = std::result::Result<T, Error>;
