//! Installing the telemetry pipeline.
//!
//! Composition roots only — `episteme-server` and `episteme-embedded`. Library
//! crates must never reach for this; they emit through the facades and let
//! whoever owns `main` decide where the data goes.

use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

/// How telemetry should be wired up.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Filter directives, e.g. `info,episteme_storage=debug`.
    pub filter: String,
    /// Emit logs as JSON rather than human-readable text.
    pub json: bool,
    /// Address to serve a Prometheus scrape endpoint on. `None` disables it.
    pub prometheus: Option<SocketAddr>,
    /// Fraction of searches re-run exhaustively to measure live recall.
    ///
    /// Zero disables it. This costs a full scan per sampled query, so it wants
    /// to stay small — but without it, production recall is unknown.
    pub recall_sample_rate: f64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            filter: "info".to_owned(),
            json: false,
            prometheus: None,
            recall_sample_rate: 0.0,
        }
    }
}

/// Live telemetry. Holding it keeps the pipeline installed.
///
/// Deliberately not `Clone`: installing twice is a bug, and the type should
/// make that awkward rather than merely discouraged.
#[derive(Debug)]
pub struct Telemetry {
    config: TelemetryConfig,
}

impl Telemetry {
    /// Install logging, tracing and metrics.
    ///
    /// Returns an error if a subscriber or recorder is already installed —
    /// which usually means an embedded caller set one up and the server tried
    /// to as well. That is worth failing on rather than silently ignoring: two
    /// pipelines means data split unpredictably between them.
    pub fn install(config: TelemetryConfig) -> Result<Self, TelemetryError> {
        let filter = EnvFilter::try_new(&config.filter)
            .map_err(|e| TelemetryError::Filter(e.to_string()))?;

        let builder = tracing_subscriber::fmt().with_env_filter(filter);
        if config.json {
            builder
                .json()
                .try_init()
                .map_err(|e| TelemetryError::AlreadyInstalled(e.to_string()))?;
        } else {
            builder
                .try_init()
                .map_err(|e| TelemetryError::AlreadyInstalled(e.to_string()))?;
        }

        if let Some(addr) = config.prometheus {
            PrometheusBuilder::new()
                .with_http_listener(addr)
                .install()
                .map_err(|e| TelemetryError::Exporter(e.to_string()))?;
        }

        describe_all();
        Ok(Self { config })
    }

    /// The configuration this pipeline was installed with.
    pub fn config(&self) -> &TelemetryConfig {
        &self.config
    }

    /// Whether this query should be re-run exhaustively to sample recall.
    ///
    /// Takes the decision as a caller-supplied draw so the choice stays
    /// deterministic under test and no RNG is pulled into this crate.
    pub fn should_sample_recall(&self, draw: f64) -> bool {
        self.config.recall_sample_rate > 0.0 && draw < self.config.recall_sample_rate
    }
}

/// Register every metric with its description so `/metrics` is self-documenting.
pub fn describe_all() {
    for (name, description) in crate::metrics_names::ALL {
        metrics::describe_histogram!(*name, *description);
    }
}

#[derive(Debug, thiserror::Error)]
/// Why installing the telemetry pipeline failed.
pub enum TelemetryError {
    #[error("invalid filter directive: {0}")]
    Filter(String),
    #[error("telemetry already installed: {0}")]
    AlreadyInstalled(String),
    #[error("exporter failed to start: {0}")]
    Exporter(String),
}

#[cfg(test)]
#[path = "init_test.rs"]
mod tests;
