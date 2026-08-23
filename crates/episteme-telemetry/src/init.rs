//! Installing the telemetry pipeline.
//!
//! Composition roots only — `episteme-server` and `episteme-embedded`. Library
//! crates must never reach for this; they emit through the `tracing` and
//! `metrics` facades and let whoever owns `main` decide where the data goes.
//!
//! The pipeline itself is [`telemetry`], the ecosystem's stack: structured
//! logging, OpenTelemetry traces and metrics, and MCAP recording, all behind
//! one builder. This module is a thin adapter that maps episteme's
//! configuration onto it and registers episteme's metric descriptions.

use std::net::SocketAddr;
use telemetry::{Telemetry as Inner, options::Environment};

/// How telemetry should be wired up.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Service name reported to the collector.
    pub service: String,
    /// Service version reported to the collector.
    pub version: String,
    /// Deployment environment, which is this stack's verbosity control.
    ///
    /// This is the stack's own verbosity control — `Development` is chatty,
    /// `Production` is not — so it replaces a log-level string rather than
    /// sitting beside one. `Jetson` exists because the stack targets it
    /// directly, which matters here: Jetson is a real deployment target for
    /// this database.
    pub environment: Environment,
    /// OTLP collector to export traces, metrics and logs to.
    ///
    /// `None` keeps everything local: console output only, nothing leaves the
    /// process.
    pub otlp: Option<SocketAddr>,
    /// Record an MCAP file at this path, for inspection in Foxglove Studio.
    pub mcap_path: Option<String>,
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
            // Off unless asked: a database should not send telemetry off the
            // machine because nobody said not to.
            otlp: None,
            mcap_path: None,
            recall_sample_rate: 0.0,
        }
    }
}

/// Live telemetry. Holding it keeps the pipeline installed and flushing.
///
/// Deliberately not `Clone`: installing twice is a bug, and the type should
/// make that awkward rather than merely discouraged.
///
/// `Debug` is written by hand and prints configuration only — the pipeline
/// itself does not implement it, and dumping its internals would put exporter
/// state in a log.
pub struct Telemetry {
    config: TelemetryConfig,
    inner: Inner,
}

impl std::fmt::Debug for Telemetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Telemetry")
            .field("service", &self.config.service)
            .field("version", &self.config.version)
            .field("environment", &self.config.environment.to_string())
            .field("otlp", &self.config.otlp)
            .finish_non_exhaustive()
    }
}

impl Telemetry {
    /// Install logging, tracing and metrics.
    ///
    /// Returns an error if a pipeline is already installed — which usually
    /// means an embedded caller set one up and the server tried to as well.
    /// Worth surfacing rather than ignoring: two pipelines split the data
    /// unpredictably between them.
    pub fn install(config: TelemetryConfig) -> Result<Self, TelemetryError> {
        // `Telemetry::new()` then `.with_service(...)` — the documented entry
        // point. `Telemetry::builder(name, version)` exists but is not what the
        // stack's own examples use.
        let mut builder = Inner::new()
            .with_service(&config.service, &config.version)
            .environment(config.environment.clone());

        // Distributed tracing is enabled only alongside a collector.
        //
        // `with_tracing()` on its own starts an exporter that tries to reach a
        // local collector, and with none running every batch fails and logs the
        // refusal — so a server with no telemetry configured would fill its own
        // console with export errors.
        if let Some(addr) = config.otlp {
            builder = builder
                .with_otlp(addr.ip().to_string(), addr.port())
                .with_tracing();
        }
        if let Some(path) = &config.mcap_path {
            builder = builder.with_mcap(path.clone());
        }

        let inner = builder
            .build()
            .map_err(|e| TelemetryError::Install(e.to_string()))?;

        describe_all();
        Ok(Self { config, inner })
    }

    /// The configuration this pipeline was installed with.
    pub fn config(&self) -> &TelemetryConfig {
        &self.config
    }

    /// Flush buffered telemetry.
    ///
    /// Worth calling before a deliberate shutdown: an exporter batches, so the
    /// last few seconds of a run are otherwise lost exactly when something has
    /// gone wrong enough to stop the process.
    pub fn flush(&self) -> Result<(), TelemetryError> {
        self.inner
            .flush()
            .map_err(|e| TelemetryError::Install(e.to_string()))
    }

    /// Whether this query should be re-run exhaustively to sample recall.
    pub fn should_sample_recall(&self, draw: f64) -> bool {
        should_sample(self.config.recall_sample_rate, draw)
    }
}

/// Whether a draw falls inside the sampling rate.
///
/// A free function rather than a method: constructing a pipeline needs a tokio
/// runtime, and a decision this simple should be testable without one.
///
/// Takes the draw from the caller so the choice stays deterministic under test
/// and no RNG is pulled into this crate.
pub fn should_sample(rate: f64, draw: f64) -> bool {
    rate > 0.0 && draw < rate
}

/// Register every metric with its description.
///
/// Without this an exported metric arrives with no help text, and whoever finds
/// it on a dashboard has to read the source to learn what it counts.
pub fn describe_all() {
    for (name, description) in crate::metrics_names::ALL {
        metrics::describe_histogram!(*name, *description);
    }
}

/// Why installing the telemetry pipeline failed.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// The pipeline could not be built or was already installed.
    #[error("telemetry: {0}")]
    Install(
        /// What the telemetry stack reported.
        String,
    ),
}

#[cfg(test)]
#[path = "init_test.rs"]
mod tests;
