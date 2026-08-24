//! How telemetry should be wired up.

use std::net::SocketAddr;
use std::path::PathBuf;
use telemetry::options::Environment;

/// Configuration for the telemetry pipeline.
///
/// Every field here is a *code-level* override, which the stack ranks above
/// `telemetry.toml` and above `TELEMETRY_*` environment variables. So a field
/// left `None` is not "off" — it is "whatever the file or the environment
/// said", which is what makes `telemetry.toml` load-bearing rather than
/// decorative.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Service name reported to the collector.
    pub service: String,
    /// Service version reported to the collector.
    pub version: String,
    /// Deployment environment, reported as a resource attribute.
    ///
    /// This is *not* the verbosity control — [`log_level`](Self::log_level) is.
    /// The environment tells a collector which deployment a span came from,
    /// which matters when staging and production export to the same place.
    pub environment: Environment,
    /// Console and export verbosity.
    ///
    /// `None` defers to `[logging] level` in `telemetry.toml`, then to the
    /// stack's own default. Per-module overrides live in the file only —
    /// `[logging.modules.<name>]` — because they are deployment tuning rather
    /// than something a flag should carry.
    pub log_level: Option<telemetry::LogLevel>,
    /// OTLP collector to export traces, metrics and logs to.
    ///
    /// `None` defers to `[telemetry.otlp]` in the config file. Note that the
    /// stack takes a *host*, not a socket address, and appends the port itself.
    pub otlp: Option<SocketAddr>,
    /// Record an MCAP file at this path, for inspection in Foxglove Studio.
    ///
    /// A `PathBuf` rather than a `String` because a path is not text: rendering
    /// one through `Display` replaces any non-UTF-8 component with U+FFFD, so
    /// the pipeline would open a different file than the one it was given and
    /// report the wrong name while doing it.
    pub mcap_path: Option<PathBuf>,
    /// Explicit path to the stack's own configuration file.
    ///
    /// `None` leaves the stack to discover `telemetry.toml` relative to the
    /// process working directory. That is fine for `cargo run` from the
    /// repository root and wrong for everything else: a deployed binary, a
    /// test binary and a benchmark all run somewhere else, find no file, and
    /// fall back to defaults — where `[telemetry] enabled` is **true**, so an
    /// OTLP exporter starts and reports a refused connection on every batch.
    ///
    /// Setting this makes the choice explicit rather than positional.
    pub config_path: Option<PathBuf>,

    /// Fraction of searches re-run exhaustively to measure live recall.
    ///
    /// Zero disables it. This costs a full scan per sampled query, so it wants
    /// to stay small — but without it, production recall is unknown.
    pub recall_sample_rate: f64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service: "episteme".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            environment: Environment::Development,
            log_level: None,
            // Off unless asked: a database should not send telemetry off the
            // machine because nobody said not to.
            otlp: None,
            mcap_path: None,
            config_path: None,
            recall_sample_rate: 0.0,
        }
    }
}

/// The host string for an OTLP endpoint.
///
/// The stack takes host and port separately and rejoins them as `{host}:{port}`
/// with no brackets, so handing it `addr.ip()` turns `[::1]:4317` into the
/// endpoint `::1:4317` — not a valid URI, and a failure that surfaces only as
/// an exporter that never connects. `SocketAddr`'s own `Display` brackets IPv6
/// correctly, so the address is rendered whole and the port trimmed back off.
pub fn otlp_host(addr: &SocketAddr) -> String {
    let rendered = addr.to_string();
    match rendered.rsplit_once(':') {
        Some((host, _port)) => host.to_owned(),
        None => rendered,
    }
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
