//! Installing the telemetry pipeline.
//!
//! Composition roots only — `episteme-server` and `episteme-embedded`. This is
//! the one place the stack is built, per CLAUDE.md rule 41.
//!
//! The pipeline itself is [`telemetry`]: structured logging, OpenTelemetry
//! traces and metrics, and MCAP recording, all behind one builder. This module
//! is a thin adapter that maps episteme's configuration onto it — it does not
//! wrap, re-implement or second-guess any part of the stack.

use crate::config::{TelemetryConfig, otlp_host};
use crate::meter::Meter;
use telemetry::{Telemetry as Inner, options::ServiceOptions};

/// Live telemetry. Holding it keeps the pipeline installed and flushing.
///
/// Deliberately not `Clone`: installing twice is a bug, and the type should
/// make that awkward rather than merely discouraged. The stack flushes and
/// shuts down its exporters when this drops.
///
/// `Debug` is written by hand and prints configuration only — the stack does
/// not implement it, and dumping its internals would put exporter state in a
/// log.
pub struct Telemetry {
    /// The configuration this pipeline was installed with.
    config: TelemetryConfig,
    /// Shared metrics recorder handed to the crates that emit.
    meter: Meter,
    /// The stack itself. Dropping it flushes and shuts down.
    inner: Inner,
}

impl std::fmt::Debug for Telemetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Telemetry")
            .field("service", &self.config.service)
            .field("version", &self.config.version)
            .field("environment", &self.config.environment.to_string())
            .field("otlp", &self.config.otlp)
            .field("recording", &self.meter.is_enabled())
            .finish_non_exhaustive()
    }
}

impl Telemetry {
    /// Install logging, tracing and metrics.
    ///
    /// Needs a tokio runtime: the stack starts exporters when it builds.
    ///
    /// # Errors
    ///
    /// Returns [`TelemetryError::Install`] if the stack could not be built — an
    /// unwritable MCAP path or an OTLP endpoint that will not resolve. That is
    /// a real failure and must not be mistaken for "already installed": a
    /// server that starts with no telemetry and no diagnostic is the case this
    /// error exists to prevent.
    pub fn install(config: TelemetryConfig) -> Result<Self, TelemetryError> {
        let mut builder = Inner::new()
            .with_service(&config.service, &config.version)
            .environment(config.environment.clone());

        // Before anything else: the stack's own config file decides whether an
        // OTLP exporter starts at all, and with no file found its default is
        // *enabled*. Pointing at the file explicitly is what keeps a binary's
        // behaviour independent of the directory it happens to run from.
        if let Some(path) = &config.config_path {
            builder = builder.with_config(utf8_arg(path, "--telemetry-config")?);
        }

        // Only override the config file when the caller actually chose a level.
        // Setting one unconditionally is what would make `[logging] level` in
        // `telemetry.toml` permanently dead, since code outranks the file.
        if let Some(level) = config.log_level {
            builder = builder.with_log_level(level);
        }

        // Distributed tracing is enabled only alongside a collector.
        //
        // `with_tracing()` on its own starts an exporter that tries to reach a
        // local collector, and with none running every batch fails and logs the
        // refusal — so a server with no telemetry configured would fill its own
        // console with export errors.
        if let Some(addr) = config.otlp {
            builder = builder
                .with_otlp(otlp_host(&addr), addr.port())
                .with_tracing();
        }
        if let Some(path) = &config.mcap_path {
            // `to_string_lossy` is the stack's own signature. Rejecting a
            // non-UTF-8 path here is better than silently recording to a
            // different one — see `mcap_path_arg`.
            builder = builder.with_mcap(utf8_arg(path, "--mcap")?);
        }

        let inner = builder
            .build()
            .map_err(|e| TelemetryError::Install(e.to_string()))?;

        // A second recorder over the same providers, rather than the one on
        // `inner`: `Metrics` records through `&mut self`, and `inner` has to
        // stay owned here so the pipeline flushes on drop. Both write to the
        // same meter and the same MCAP file.
        let metrics = telemetry::Metrics::new(
            ServiceOptions::new(&config.service, &config.version),
            inner.mcap_writer(),
            inner.meter_provider(),
        )
        .map_err(|e| TelemetryError::Install(e.to_string()))?;

        Ok(Self {
            config,
            meter: Meter::new(metrics),
            inner,
        })
    }

    /// The configuration this pipeline was installed with.
    pub fn config(&self) -> &TelemetryConfig {
        &self.config
    }

    /// A handle the emitting crates record through.
    ///
    /// Cheap to clone, and every clone reaches the same recorder.
    pub fn meter(&self) -> Meter {
        self.meter.clone()
    }

    /// Flush buffered telemetry.
    ///
    /// Worth calling before a deliberate shutdown: an exporter batches, so the
    /// last few seconds of a run are otherwise lost exactly when something has
    /// gone wrong enough to stop the process.
    ///
    /// # Errors
    ///
    /// Returns [`TelemetryError::Install`] carrying whatever the stack reported
    /// — usually a collector that has stopped accepting.
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

/// Render a path for the stack, refusing one that would not survive it.
///
/// The stack takes strings. A path that is not valid UTF-8 cannot be handed
/// over without `to_string_lossy` replacing bytes with U+FFFD, at which point
/// the stack opens a *different* file than the one requested and reports the
/// wrong name. Refusing is the only honest option.
pub fn utf8_arg(path: &std::path::Path, flag: &str) -> Result<String, TelemetryError> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        TelemetryError::Install(format!(
            "{flag} {}: path is not valid UTF-8, and the telemetry stack takes \
             a string — it would silently use a different path",
            path.display()
        ))
    })
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

/// Why installing the telemetry pipeline failed.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// The pipeline could not be built.
    #[error("telemetry: {0}")]
    Install(
        /// What the telemetry stack reported.
        String,
    ),
}

#[cfg(test)]
#[path = "init_test.rs"]
mod tests;
