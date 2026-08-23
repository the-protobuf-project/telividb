//! Server errors, and how they reach a client.

/// Failures starting or running the server.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The listen address could not be bound.
    #[error("bind {addr}: {source}")]
    Bind {
        /// Address that could not be bound.
        addr: std::net::SocketAddr,
        /// Why the bind failed — usually a port already in use.
        source: std::io::Error,
    },

    /// The reflection service could not be built from the descriptor set.
    #[error("reflection: {0}")]
    Reflection(
        /// What the reflection builder reported.
        String,
    ),

    /// Telemetry could not be installed.
    #[error("telemetry: {0}")]
    Telemetry(
        /// What the telemetry installer reported.
        String,
    ),

    /// The transport stopped unexpectedly.
    #[error("transport: {0}")]
    Transport(
        /// What tonic reported.
        String,
    ),
}

/// Convenience alias for a server result.
pub type Result<T> = std::result::Result<T, Error>;

/// Map a domain error onto a gRPC status.
///
/// Mapped in one place, deliberately. Scattering conversions is how an internal
/// message ends up in a client-visible status, and how a caller learns that a
/// resource exists by reading the difference between two error strings.
pub fn to_status(error: &episteme_core::Error) -> tonic::Status {
    use episteme_core::Error as E;
    match error {
        E::DimMismatch { .. } | E::ZeroDim | E::InvalidSpan { .. } | E::NonFinite { .. } => {
            tonic::Status::invalid_argument(error.to_string())
        }
        E::InvalidResourceName { .. } => tonic::Status::invalid_argument(error.to_string()),
        // Deliberately opaque: a malformed index is an operator problem, and its
        // detail belongs in the log rather than in a response.
        E::MalformedIndex { .. } => {
            tracing::error!(%error, "malformed index");
            tonic::Status::internal("index could not be read")
        }
    }
}
