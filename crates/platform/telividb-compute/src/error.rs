//! What can go wrong inside the runtime.

/// A failure allocating on, or computing with, a device.
#[derive(Debug)]
pub enum Error {
    /// No backend of the requested kind could be initialised.
    ///
    /// Ordinary rather than exceptional: a machine without a GPU reports this
    /// and the caller falls back to the CPU, which is always available.
    BackendUnavailable {
        /// Which backend was asked for.
        kind: &'static str,
    },

    /// A device allocation failed.
    ///
    /// Reported rather than aborting, because it is the one device failure a
    /// caller can act on — by building a smaller index, or on the host.
    Allocation {
        /// Bytes that could not be allocated.
        bytes: usize,
    },

    /// Two tensors could not take part in an operation together.
    ShapeMismatch {
        /// What the operation required.
        expected: String,
        /// What it was given.
        actual: String,
    },

    /// ggml refused an operation and said why.
    Runtime {
        /// The operation that failed.
        op: &'static str,
        /// What went wrong.
        reason: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BackendUnavailable { kind } => {
                write!(f, "no {kind} backend is available in this build")
            }
            Error::Allocation { bytes } => {
                write!(f, "could not allocate {bytes} bytes on the device")
            }
            Error::ShapeMismatch { expected, actual } => {
                write!(f, "shape mismatch: expected {expected}, got {actual}")
            }
            Error::Runtime { op, reason } => write!(f, "{op} failed: {reason}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias for a runtime result.
pub type Result<T> = std::result::Result<T, Error>;
