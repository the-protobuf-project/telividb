//! The gRPC composition root.
//!
//! One of two roots — `telividb-embedded` is the other, and neither owns the
//! other. This is where adapters are chosen and wired; nothing below this line
//! knows a server exists.
//!
//! What the server adds beyond routing:
//!
//! - **Health and reflection** from the first release. Reflection is what lets
//!   `grpcurl`, generic clients and MCP bridges introspect the API without
//!   being shipped the protos.
//! - **gRPC-web**, because browsers cannot speak native gRPC.
//! - **Telemetry**, installed exactly once here rather than in any library.
//!
//! It deliberately does *not* enforce authorization. That lives in the query
//! planner, so the embedded path cannot bypass it — see CLAUDE.md.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod announce;
pub mod args;
pub mod config;
pub mod error;
pub mod serve;
pub mod services;

pub use config::ServerConfig;
pub use error::{Error, Result};
pub use serve::serve;
