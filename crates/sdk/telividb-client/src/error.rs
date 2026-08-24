//! What can go wrong, in terms a caller can act on.

/// A failure talking to a telividb server.
///
/// Modelled on what the caller would *do* about it rather than on gRPC's
/// status codes: a bare `tonic::Status` forces every caller to match on
/// integers and to know which code the server chose for which condition.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The server could not be reached, or the connection was lost.
    #[error("transport: {0}")]
    Transport(
        /// The underlying connection failure.
        #[from]
        tonic::transport::Error,
    ),

    /// The named resource does not exist.
    ///
    /// Separated from other failures because it is routinely *expected* — a
    /// `get` on something not yet created is a normal outcome, not an error
    /// worth logging.
    #[error("not found: {name}")]
    NotFound {
        /// Resource name that was not found.
        name: String,
    },

    /// A resource with this name already exists.
    #[error("already exists: {name}")]
    AlreadyExists {
        /// Resource name that was already taken.
        name: String,
    },

    /// The request was rejected as invalid.
    ///
    /// Carries the server's own message: it knows which field was wrong and a
    /// generic "invalid argument" would throw that away.
    #[error("invalid request: {message}")]
    InvalidArgument {
        /// What the server said was wrong.
        message: String,
    },

    /// The caller is not permitted to do this.
    ///
    /// Distinguishable from an empty result on purpose. A denial reported as
    /// "nothing found" leaves a caller unable to tell a missing row from one
    /// they may not see, which is the confusion rules 27 and 49 exist to
    /// prevent.
    #[error("permission denied: {message}")]
    PermissionDenied {
        /// The server's explanation, as far as it is willing to give one.
        message: String,
    },

    /// Anything else the server reported.
    #[error("server error ({code:?}): {message}")]
    Server {
        /// gRPC status code, kept so an unmatched condition is still legible.
        code: tonic::Code,
        /// The server's message.
        message: String,
    },

    /// A response did not carry what its contract promises.
    ///
    /// Reported rather than papered over with a default: a vector whose byte
    /// length disagrees with its declared width means client and server
    /// disagree about the format, and inventing a value would hide that.
    #[error("malformed response: {what}")]
    Malformed {
        /// Which part of the response could not be read.
        what: String,
    },
}

impl From<tonic::Status> for Error {
    /// Sort a status into the variants above, keeping the server's message.
    fn from(status: tonic::Status) -> Self {
        let message = status.message().to_owned();
        match status.code() {
            tonic::Code::NotFound => Error::NotFound { name: message },
            tonic::Code::AlreadyExists => Error::AlreadyExists { name: message },
            tonic::Code::InvalidArgument => Error::InvalidArgument { message },
            tonic::Code::PermissionDenied | tonic::Code::Unauthenticated => {
                Error::PermissionDenied { message }
            }
            code => Error::Server { code, message },
        }
    }
}

/// Convenience alias for an SDK result.
pub type Result<T> = std::result::Result<T, Error>;
