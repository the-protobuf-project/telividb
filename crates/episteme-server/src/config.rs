//! How the server is configured.

use std::net::SocketAddr;
use std::path::PathBuf;

/// Listen addresses and feature toggles.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to serve gRPC on.
    pub addr: SocketAddr,
    /// Serve gRPC-web alongside native gRPC on the same port.
    ///
    /// One port is materially simpler to deploy, firewall and reason about, and
    /// the embedded UI needs gRPC-web regardless.
    pub grpc_web: bool,
    /// Serve the gRPC reflection service.
    ///
    /// On by default: without it a client must already hold the protos, which
    /// defeats generic tooling.
    pub reflection: bool,
    /// Deployment environment, which is the telemetry stack's verbosity
    /// control: `development` is chatty, `production` is not.
    pub environment: String,
    /// OTLP collector to export traces, metrics and logs to.
    ///
    /// `None` keeps everything on the console. A daemon should set this:
    /// under launchd or systemd stdout goes nowhere unless the supervisor is
    /// capturing it, so a server that only writes to the console appears to
    /// emit nothing at all.
    pub otlp_addr: Option<SocketAddr>,

    /// Record an MCAP file at this path, for inspection in Foxglove Studio.
    pub mcap_path: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:7700"
                .parse()
                .expect("literal is a valid address"),
            grpc_web: true,
            reflection: true,
            environment: "development".to_owned(),
            // Off unless asked: a database should not open a port nobody
            // requested.
            otlp_addr: None,
            mcap_path: None,
        }
    }
}

impl ServerConfig {
    /// Bind to `addr`, keeping every other default.
    pub fn at(addr: SocketAddr) -> Self {
        Self {
            addr,
            ..Default::default()
        }
    }
}
