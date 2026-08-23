//! How the server is configured.

use std::net::SocketAddr;

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
    /// Filter directives for logs, e.g. `info,episteme_storage=debug`.
    pub log_filter: String,
    /// Emit logs as JSON rather than human-readable text.
    pub log_json: bool,
    /// Address for a Prometheus scrape endpoint. `None` disables it.
    pub metrics_addr: Option<SocketAddr>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:7700"
                .parse()
                .expect("literal is a valid address"),
            grpc_web: true,
            reflection: true,
            log_filter: "info".to_owned(),
            log_json: false,
            // Off unless asked: a database should not open a port nobody
            // requested.
            metrics_addr: None,
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
