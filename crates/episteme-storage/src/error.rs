//! Storage errors.

/// Failures reading or writing durable state.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Domain(#[from] episteme_core::Error),

    /// The magic bytes did not match. This is not our file.
    #[error("bad magic: expected {expected:?}, found {found:?}")]
    BadMagic { expected: [u8; 4], found: [u8; 4] },

    /// A format written by a newer episteme. Refuse rather than guess — a
    /// misread segment is worse than a clear failure. See CLAUDE.md
    /// invariant 4.
    #[error("unsupported {what} format version {found}; this build reads up to {supported}")]
    UnsupportedVersion {
        what: &'static str,
        found: u16,
        supported: u16,
    },

    /// A checksum did not match the bytes it covers.
    #[error("checksum mismatch in {what}: expected {expected:#010x}, computed {computed:#010x}")]
    Corrupt {
        what: &'static str,
        expected: u32,
        computed: u32,
    },

    /// Fewer bytes than the structure requires.
    #[error("truncated {what}: needed {needed} bytes, found {found}")]
    Truncated {
        what: &'static str,
        needed: usize,
        found: usize,
    },

    /// A segment was written under a different schema than the one in force.
    ///
    /// Refused rather than reconciled: reading columns under the wrong schema
    /// puts values in the wrong places and reports nothing.
    #[error("schema drift: segment was written under {segment}, collection is now {current}")]
    SchemaDrift { segment: String, current: String },

    /// Vectors in a field were produced by a different model.
    #[error("model drift: field holds vectors from {segment}, configured model is {current}")]
    ModelDrift { segment: String, current: String },

    /// A PQ configuration that cannot describe a vector.
    ///
    /// Refused rather than padded: a silent pad makes the final subspace
    /// partly meaningless, and the resulting recall loss is very hard to
    /// attribute back to here.
    #[error("invalid pq shape: {m} subspaces do not divide {dim} dimensions evenly")]
    InvalidPqShape { dim: usize, m: usize },

    /// A vector or code run did not match the codebook it was used with.
    #[error("pq length mismatch: expected {expected}, got {actual}")]
    PqDimMismatch { expected: usize, actual: usize },

    /// A byte did not correspond to any known enum variant.
    #[error("unknown {what} discriminant {value}")]
    UnknownDiscriminant { what: &'static str, value: u8 },
}

pub type Result<T> = std::result::Result<T, Error>;
