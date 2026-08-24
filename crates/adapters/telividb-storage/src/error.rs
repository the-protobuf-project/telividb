//! Storage errors.

/// Failures reading or writing durable state.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A filesystem operation failed.
    #[error("io: {0}")]
    Io(
        /// The underlying filesystem failure.
        #[from]
        std::io::Error,
    ),

    /// A domain invariant was violated beneath a storage operation.
    #[error(transparent)]
    Domain(
        /// The underlying domain-invariant failure.
        #[from]
        telividb_core::Error,
    ),

    /// A `redb`-backed metadata store failed.
    ///
    /// `redb::Error` is itself a superset of every operation-specific error
    /// the crate defines (database, transaction, table, storage), so this one
    /// variant covers all of them.
    #[error("redb: {0}")]
    Redb(
        /// The underlying redb failure.
        #[from]
        redb::Error,
    ),

    /// The magic bytes did not match. This is not our file.
    #[error("bad magic: expected {expected:?}, found {found:?}")]
    BadMagic {
        /// Magic this reader expects.
        expected: [u8; 4],
        /// Magic actually present at the head of the file.
        found: [u8; 4],
    },

    /// A format written by a newer telividb. Refuse rather than guess — a
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

    /// A PQ codebook was trained on too few vectors to be meaningful.
    ///
    /// With no training vectors at all, seeding returns zeros and the update
    /// loop exits before it runs, so `train` used to succeed with a degenerate
    /// codebook: every row then encodes to code 0, every distance is identical,
    /// and the scan tier ranks nothing — silently, with no error anywhere and
    /// full recall loss.
    #[error(
        "pq codebook needs at least {needed} training vectors per subspace, got {found}; \
         a codebook trained on fewer cannot distinguish rows"
    )]
    PqTrainingTooSmall {
        /// Vectors required — one per centroid.
        needed: usize,
        /// Vectors actually supplied.
        found: usize,
    },

    /// A vector or code run did not match the codebook it was used with.
    #[error("pq length mismatch: expected {expected}, got {actual}")]
    PqDimMismatch {
        /// Length the codebook requires.
        expected: usize,
        /// Length actually supplied.
        actual: usize,
    },

    /// A field was opened under different terms than it was created with.
    ///
    /// Refused rather than reconciled: the wrong dimension reinterprets the
    /// field's bytes, and the wrong metric ranks correctly-read vectors
    /// wrongly. Both fail silently, so both are caught here.
    #[error("field {what} mismatch: stored {stored}, requested {requested}")]
    FieldMismatch {
        /// Which property disagreed.
        what: &'static str,
        /// What the field was created with.
        stored: String,
        /// What the caller asked for.
        requested: String,
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
