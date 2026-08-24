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
pub fn to_status(error: &telividb_core::Error) -> tonic::Status {
    use telividb_core::Error as E;
    match error {
        E::DimMismatch { .. } | E::ZeroDim | E::InvalidSpan { .. } | E::NonFinite { .. } => {
            tonic::Status::invalid_argument(error.to_string())
        }
        E::InvalidResourceName { .. } => tonic::Status::invalid_argument(error.to_string()),
        // Deliberately opaque: a malformed index is an operator problem, and its
        // detail belongs in the log rather than in a response.
        E::MalformedIndex { .. } => {
            telividb_telemetry::logger::error!("malformed index: {error}");
            tonic::Status::internal("index could not be read")
        }
        // Same reasoning: a `GraphStore` adapter failure (a redb I/O error, a
        // corrupt key) is an operator problem, not something the caller did.
        E::GraphStore { .. } => {
            telividb_telemetry::logger::error!("graph store: {error}");
            tonic::Status::internal("graph could not be read")
        }
        // Same reasoning again, for the document service's `PointStore`.
        E::PointStore { .. } => {
            telividb_telemetry::logger::error!("point store: {error}");
            tonic::Status::internal("point store could not be read")
        }
        // A device allocation or tensor failure is an operator problem too —
        // and its detail names GPU internals a caller has no use for.
        E::GpuIndex { .. } => {
            telividb_telemetry::logger::error!("gpu index: {error}");
            tonic::Status::internal("index could not be searched")
        }
    }
}

/// Map a `telividb-storage` failure onto a gRPC status.
///
/// A distinct function rather than a `to_status` overload: `PointsSvc` calls
/// straight into `RedbPointStore`'s own `create`/`delete` and the factory
/// that opens it, both of which return `telividb_storage::Error` — a
/// different type from the `telividb_core::Error` the port methods
/// (`get`/`list`) surface, which `to_status` already handles. Same posture
/// either way: opaque to the caller, logged for the operator.
pub fn storage_status(error: &telividb_storage::Error) -> tonic::Status {
    telividb_telemetry::logger::error!("storage: {error}");
    tonic::Status::internal("storage could not be read")
}
