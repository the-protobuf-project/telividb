//! Storage errors.

/// Failures reading or writing durable state.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(
        /// The underlying filesystem failure.
        #[from]
        std::io::Error,
    ),

    #[error(transparent)]
    Domain(
        /// The underlying domain-invariant failure.
        #[from]
        episteme_core::Error,
    ),

    /// The magic bytes did not match. This is not our file.
    #[error("bad magic: expected {expected:?}, found {found:?}")]
    BadMagic {
        /// Magic this reader expects.
        expected: [u8; 4],
        /// Magic actually present at the head of the file.
        found: [u8; 4],
    },

    /// A format written by a newer episteme. Refuse rather than guess — a
    /// misread segment is worse than a clear failure. See CLAUDE.md
    /// invariant 4.
    #[error("unsupported {what} format version {found}; this build reads up to {supported}")]
    UnsupportedVersion {
        /// Which structure carried the version.
        what: &'static str,
        /// Version the file declares.
        found: u16,
        /// Highest version this build can read.
        supported: u16,
    },

    /// A checksum did not match the bytes it covers.
    #[error("checksum mismatch in {what}: expected {expected:#010x}, computed {computed:#010x}")]
    Corrupt {
        /// Which structure failed its checksum.
        what: &'static str,
        /// Checksum recorded in the file.
        expected: u32,
        /// Checksum computed over the bytes actually read.
        computed: u32,
    },

    /// Fewer bytes than the structure requires.
    #[error("truncated {what}: needed {needed} bytes, found {found}")]
    Truncated {
        /// Which structure was short.
        what: &'static str,
        /// Bytes the structure requires.
        needed: usize,
        /// Bytes actually available.
        found: usize,
    },

    /// A segment was written under a different schema than the one in force.
    ///
    /// Refused rather than reconciled: reading columns under the wrong schema
    /// puts values in the wrong places and reports nothing.
    #[error("schema drift: segment was written under {segment}, collection is now {current}")]
    SchemaDrift {
        /// Short fingerprint the segment was written under.
        segment: String,
        /// Short fingerprint the collection now declares.
        current: String,
    },

    /// Vectors in a field were produced by a different model.
    #[error("model drift: field holds vectors from {segment}, configured model is {current}")]
    ModelDrift {
        /// Short fingerprint of the model that produced the stored vectors.
        segment: String,
        /// Short fingerprint of the model now configured for the field.
        current: String,
    },

    /// A PQ configuration that cannot describe a vector.
    ///
    /// Refused rather than padded: a silent pad makes the final subspace
    /// partly meaningless, and the resulting recall loss is very hard to
    /// attribute back to here.
    #[error("invalid pq shape: {m} subspaces do not divide {dim} dimensions evenly")]
    InvalidPqShape {
        /// Vector width being divided.
        dim: usize,
        /// Requested subspace count, which does not divide `dim` evenly.
        m: usize,
    },

    /// A vector or code run did not match the codebook it was used with.
    #[error("pq length mismatch: expected {expected}, got {actual}")]
    PqDimMismatch {
        /// Length the codebook requires.
        expected: usize,
        /// Length actually supplied.
        actual: usize,
    },

    /// A byte did not correspond to any known enum variant.
    #[error("unknown {what} discriminant {value}")]
    UnknownDiscriminant {
        /// Which enum the byte was meant to select.
        what: &'static str,
        /// The unrecognised byte.
        value: u8,
    },
}

/// Convenience alias for a storage-layer result.
pub type Result<T> = std::result::Result<T, Error>;
