//! One error enum for the domain layer.

/// Errors that arise from domain invariants, independent of any adapter.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A vector's length did not match the field's declared dimension.
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimMismatch { expected: usize, actual: usize },

    /// A span's end preceded its start.
    #[error("invalid span: start {start_ms}ms is after end {end_ms}ms")]
    InvalidSpan { start_ms: u64, end_ms: u64 },

    /// A vector contained a non-finite value; these poison distance kernels.
    #[error("vector contains a non-finite value at index {index}")]
    NonFinite { index: usize },

    /// A dimension of zero is never meaningful.
    #[error("dimension must be non-zero")]
    ZeroDim,

    /// A serialized index was malformed.
    ///
    /// Index files are untrusted input the moment an archive arrives from
    /// elsewhere, so a lying length field must surface here rather than as a
    /// panic or an out-of-bounds read.
    #[error("malformed index: {reason}")]
    MalformedIndex { reason: &'static str },

    /// A resource name or template was not well-formed.
    #[error("invalid resource name {name:?}: {reason}")]
    InvalidResourceName { name: String, reason: &'static str },
}

pub type Result<T> = std::result::Result<T, Error>;
