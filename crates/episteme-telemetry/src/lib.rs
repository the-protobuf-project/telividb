//! Telemetry vocabulary and wiring.
//!
//! # Layering
//!
//! Library crates depend on this for **facades only** — `tracing` spans and
//! `metrics` counters that compile to near-nothing when no subscriber or
//! recorder is installed. No async runtime, no exporter, no configuration
//! reaches `episteme-storage` or `episteme-index`, which keeps them
//! synchronous, benchable and embeddable.
//!
//! Composition roots (`episteme-server`, `episteme-embedded`) enable the
//! `subscriber` feature and install the actual pipeline exactly once.
//!
//! # Why a shared vocabulary
//!
//! Field and metric names are **constants, not string literals at call sites**.
//! A span that says `collection` in one crate and `collection_name` in another
//! cannot be correlated, and the mistake is invisible until someone tries to
//! query the data. Everything emitted anywhere in episteme draws its keys from
//! [`fields`] and its metric names from [`metrics_names`].
//!
//! # Two rules that are not stylistic
//!
//! **Cardinality**: metric labels must be bounded. A segment id or a resource
//! name as a label is a time-series explosion that takes the monitoring system
//! down with it. High-cardinality facts belong on spans, which are sampled.
//! See [`fields::LABEL_SAFE`].
//!
//! **Confidentiality**: telemetry is an exfiltration path that bypasses every
//! control in the security model. Query vectors, payload contents and vault
//! names must never be emitted. See [`redact`].
#![forbid(unsafe_code)]

pub mod fields;
pub mod metrics_names;
pub mod redact;

#[cfg(feature = "subscriber")]
pub mod init;

#[cfg(feature = "subscriber")]
pub use init::{Telemetry, TelemetryConfig};
