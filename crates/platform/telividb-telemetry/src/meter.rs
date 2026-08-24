//! A shared handle onto the stack's metrics recorder.
//!
//! # Why this wrapper exists
//!
//! [`telemetry::Metrics`] records through `&mut self` and the stack exposes no
//! global for it — unlike `logger::`, which is installed once and reachable
//! from anywhere. Every telividb call site that records a metric sits behind
//! `&self`: a search runs concurrently on a shared index, and a manifest
//! publishes without owning the pipeline. Those call sites cannot take `&mut`.
//!
//! So the recorder is shared, and sharing it needs a lock. That is the whole
//! job of this type: hold the stack's recorder, hand out cheap clones, and keep
//! the lock scope down to the single `record` call inside it.
//!
//! # What it costs
//!
//! One uncontended mutex acquisition per metric emission. On the search path
//! that is three per query. A [`Meter::disabled`] handle takes no lock at all
//! and compiles to a branch, which is what every test, benchmark and embedded
//! caller gets by default.

use std::sync::{Arc, Mutex};
use telemetry::metrics::RecordMetrics;

/// A cloneable handle onto the metrics recorder.
///
/// [`Default`] is disabled rather than enabled: a library crate constructing
/// one of its own types must not start recording into a pipeline nobody asked
/// for, and a test must not need a tokio runtime to build a `WalWriter`.
#[derive(Clone, Default)]
pub struct Meter(Option<Arc<Mutex<telemetry::Metrics>>>);

impl std::fmt::Debug for Meter {
    /// Prints whether recording is on, never the recorder's internals.
    ///
    /// The recorder holds instrument caches and an MCAP writer; dumping those
    /// into a log would put exporter state where invariant 28 says it must not
    /// go.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Meter").field(&self.is_enabled()).finish()
    }
}

impl Meter {
    /// A handle that records nothing.
    ///
    /// Not an error case: it is what a library crate holds until a composition
    /// root gives it something better.
    pub fn disabled() -> Self {
        Self(None)
    }

    /// Wrap the stack's recorder so it can be shared across call sites.
    pub fn new(metrics: telemetry::Metrics) -> Self {
        Self(Some(Arc::new(Mutex::new(metrics))))
    }

    /// Whether emissions through this handle reach the pipeline.
    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    /// Add to a counter.
    pub fn counter(&self, name: &'static str, value: f64) {
        self.with(|m| m.counter(name, value));
    }

    /// Record a sample into a histogram.
    pub fn histogram(&self, name: &'static str, value: f64) {
        self.with(|m| m.histogram(name, value));
    }

    /// Set a gauge.
    pub fn gauge(&self, name: &'static str, value: f64) {
        self.with(|m| m.gauge(name, value));
    }

    /// Record every field of a [`RecordMetrics`] type at once.
    ///
    /// This is the stack's richer path: the derive carries each field's name,
    /// description and instrument kind, so the collector receives documented
    /// metrics rather than bare numbers.
    pub fn record<T: RecordMetrics>(&self, model: &T) {
        self.with(|m| m.record(model));
    }

    /// Run `f` against the recorder, discarding both the lock and the result.
    ///
    /// Failing to record a metric must never fail the operation being measured
    /// — a write that succeeded and then reported an export error would be
    /// worse than the missing data point. A poisoned lock is recovered rather
    /// than propagated for the same reason: the recorder holds no invariant a
    /// panic could have broken, only instrument caches.
    ///
    /// Generic in the return type so the stack's error type is never named
    /// here: it is `anyhow`, which CLAUDE.md keeps out of library crates.
    fn with<T>(&self, f: impl FnOnce(&mut telemetry::Metrics) -> T) {
        if let Some(metrics) = &self.0 {
            let mut guard = metrics.lock().unwrap_or_else(|e| e.into_inner());
            let _ = f(&mut guard);
        }
    }
}

#[cfg(test)]
#[path = "meter_test.rs"]
mod tests;
