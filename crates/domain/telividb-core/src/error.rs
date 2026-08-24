//! One error enum for the domain layer.

/// Errors that arise from domain invariants, independent of any adapter.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A vector's length did not match the field's declared dimension.
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimMismatch {
        /// Width the field's schema declares.
        expected: usize,
        /// Width the supplied vector actually had.
        actual: usize,
    },

    /// A span's end preceded its start.
    #[error("invalid span: start {start_ms}ms is after end {end_ms}ms")]
    InvalidSpan {
        /// Start offset, in milliseconds from the beginning of the source.
        start_ms: u64,
        /// End offset, which was before `start_ms`.
        end_ms: u64,
    },

    /// A vector contained a non-finite value; these poison distance kernels.
    #[error("vector contains a non-finite value at index {index}")]
    NonFinite {
        /// Position of the first non-finite component.
        index: usize,
    },

    /// A dimension of zero is never meaningful.
    #[error("dimension must be non-zero")]
    ZeroDim,

    /// A serialized index was malformed.
    ///
    /// Index files are untrusted input the moment an archive arrives from
    /// elsewhere, so a lying length field must surface here rather than as a
    /// panic or an out-of-bounds read.
    #[error("malformed index: {reason}")]
    MalformedIndex {
        /// What about the encoding was wrong.
        reason: &'static str,
    },

    /// A resource name or template was not well-formed.
    #[error("invalid resource name {name:?}: {reason}")]
    InvalidResourceName {
        /// The offending name or template.
        name: String,
        /// Which rule it broke.
        reason: &'static str,
    },

    /// A [`GraphStore`](crate::ports::GraphStore) adapter could not read or
    /// decode its edge records.
    ///
    /// `String` rather than `&'static str`: unlike `MalformedIndex`, the
    /// underlying failure comes from an adapter's own storage engine (a redb
    /// I/O error, a corrupt key), whose message is not one of a fixed set
    /// this crate can name in advance.
    #[error("graph store: {reason}")]
    GraphStore {
        /// What the adapter reported, as it reported it.
        reason: String,
    },

    /// A [`PointStore`](crate::ports::PointStore) adapter could not read or
    /// decode its point records. Same reasoning as `GraphStore` above.
    #[error("point store: {reason}")]
    PointStore {
        /// What the adapter reported, as it reported it.
        reason: String,
    },
}

/// Convenience alias for a domain-layer result.
pub type Result<T> = std::result::Result<T, Error>;
