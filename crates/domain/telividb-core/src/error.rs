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

    /// A GPU index failed to build, load, or score.
    ///
    /// Covers device allocation, GGUF decoding and tensor arithmetic alike:
    /// the underlying message comes from `candle`, which this crate cannot
    /// name as a type because dependencies point inward (invariant 14).
    #[error("gpu index: {reason}")]
    GpuIndex {
        /// What the tensor runtime reported, as it reported it.
        reason: String,
    },
}

/// Convenience alias for a domain-layer result.
pub type Result<T> = std::result::Result<T, Error>;
