//! How the server is configured.

use std::net::SocketAddr;
use std::path::PathBuf;
use telividb_telemetry::{Environment, LogLevel};

/// Listen addresses and feature toggles.
#[derive(Debug)]
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
    /// Deployment environment, reported to the collector as a resource
    /// attribute so staging and production stay distinguishable when they
    /// export to the same place.
    ///
    /// Typed rather than a `String`: as text, an embedded caller writing
    /// `ServerConfig { environment: "prod".to_owned(), .. }` bypassed argument
    /// parsing entirely and fell through to `Development`, so a production
    /// deployment reported `prod` while behaving as development. The enum makes
    /// that unrepresentable and moves the one string conversion into
    /// [`crate::args::parse`].
    pub environment: Environment,

    /// Console and export verbosity.
    ///
    /// `None` defers to `[logging] level` in `telemetry.toml`, which is also
    /// where per-module overrides live. A flag that always won would make that
    /// file's logging section permanently dead.
    pub log_level: Option<LogLevel>,
    /// OTLP collector to export traces, metrics and logs to.
    ///
    /// `None` keeps everything on the console. A daemon should set this:
    /// under launchd or systemd stdout goes nowhere unless the supervisor is
    /// capturing it, so a server that only writes to the console appears to
    /// emit nothing at all.
    pub otlp_addr: Option<SocketAddr>,

    /// Record an MCAP file at this path, for inspection in Foxglove Studio.
    pub mcap_path: Option<PathBuf>,

    /// Explicit path to `telemetry.toml`.
    ///
    /// `None` leaves the stack to discover it relative to the working
    /// directory, which is right for `cargo run` from the repository root and
    /// wrong for a deployed binary — it finds no file, falls back to defaults
    /// where the OTLP pipeline is *enabled*, and reports a refused connection
    /// on every batch. A daemon should set this.
    pub telemetry_config: Option<PathBuf>,

    /// An external stop signal, in addition to ctrl-c.
    ///
    /// A server that can only be stopped by a process signal cannot be run
    /// twice in one process, which makes restart behaviour untestable and
    /// makes embedding awkward — an embedded caller has no signal to send.
    /// Resolving this future begins a graceful shutdown just as ctrl-c does.
    pub shutdown: Option<tokio::sync::oneshot::Receiver<()>>,

    /// GGUF embedding model to hold resident, so callers may send text
    /// instead of vectors.
    ///
    /// `None` means the server accepts pre-computed vectors only, and refuses
    /// text with a message naming this flag. Off by default because a model is
    /// hundreds of megabytes of residency a vector-only deployment would never
    /// use — and because loading one is the sort of thing an operator should
    /// have asked for.
    pub model_path: Option<PathBuf>,

    /// Name the model is served under.
    ///
    /// A label; the model's real identity is the digest of its file (rule 12),
    /// which the server computes on load rather than trusting from here.
    pub model_name: String,

    /// Where collection data lives, one subdirectory per collection.
    ///
    /// Relative by default, matching `telemetry_config`'s CWD-relative
    /// discovery — right for `cargo run` from the repository root, wrong for
    /// a deployed binary, which should set this explicitly.
    pub data_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:7700"
                .parse()
                .expect("literal is a valid address"),
            grpc_web: true,
            reflection: true,
            environment: Environment::Development,
            log_level: None,
            // Off unless asked: a database should not open a port nobody
            // requested.
            otlp_addr: None,
            mcap_path: None,
            telemetry_config: None,
            shutdown: None,
            model_path: None,
            model_name: "default".to_owned(),
            data_dir: PathBuf::from("./data"),
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
