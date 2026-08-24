//! Telemetry vocabulary and wiring.
//!
//! # Layering
//!
//! [`telemetry`] — the ecosystem's stack — is the pipeline for logging,
//! tracing, metrics and MCAP recording, per CLAUDE.md rule 41. There is no
//! facade in front of it and no second pipeline behind it. Crates that emit
//! depend on the stack directly and log through [`logger`]; this crate carries
//! the shared vocabulary, the redaction rules, and the one place the pipeline
//! is built.
//!
//! Emission is safe everywhere: `logger::` is a no-op until a composition root
//! calls [`Telemetry::install`], and a [`Meter`] defaults to disabled. A
//! library crate, a benchmark and a unit test therefore pay nothing for being
//! instrumented, and none of them needs a runtime.
//!
//! # Why a shared vocabulary
//!
//! Field and metric names are **constants, not string literals at call sites**.
//! A record that says `collection` in one crate and `collection_name` in
//! another cannot be correlated, and the mistake is invisible until someone
//! tries to query the data. Everything emitted anywhere in telividb draws its
//! keys from [`fields`] and its metric names from [`metrics_names`].
//!
//! # Two rules that are not stylistic
//!
//! **Cardinality**: metric names must be bounded. A segment id or a resource
//! name folded into a metric name is a time-series explosion that takes the
//! monitoring system down with it. High-cardinality facts belong on a log
//! record's structured data, which is sampled. See [`fields::LABEL_SAFE`].
//!
//! **Confidentiality**: telemetry is an exfiltration path that bypasses every
//! control in the security model. Query vectors, payload contents and vault
//! names must never be emitted. See [`redact`].
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod catalogue;
pub mod config;
pub mod fields;
pub mod init;
pub mod meter;
pub mod metrics_names;
pub mod redact;

pub use config::TelemetryConfig;
pub use init::{Telemetry, TelemetryError, should_sample};
pub use meter::Meter;

/// The telemetry stack's logging macros, re-exported.
///
/// Everything that logs uses these — `logger::info!("...")` — because they
/// carry file and line, attach structured data via `.with_data(&value)`, and
/// reach the console and the OTLP log pipeline through the one stack.
///
/// They emit nothing until a composition root installs the pipeline, which is
/// what makes them safe to call from a library crate.
pub use telemetry::logger;

/// Deployment environments the stack recognises.
pub use telemetry::options::Environment;

/// Verbosity levels the stack recognises.
///
/// Re-exported so a composition root can set one without depending on the
/// stack's module layout, and so `--log-level` has a type to parse into.
pub use telemetry::LogLevel;

/// The instrument kind a metric is recorded through.
pub use telemetry::metrics::MetricType;
