//! What the server says about itself at startup and shutdown.
//!
//! Split from `serve.rs` because it is a different concern: that file starts a
//! server, this one explains what was started. The alternative it replaced was
//! a server that logged one line and then appeared silent, with no indication
//! that the instrumentation existed, that metrics were off, or where a log file
//! would be if there were one.

use crate::config::ServerConfig;
use telividb_telemetry::{fields, logger, redact};

/// Say where every stream of telemetry goes, at startup, every time.
///
/// The alternative is what this replaced: a server that logs one line and then
/// appears silent, with no indication that the instrumentation exists, that
/// metrics are off, or where a log file would be if there were one.
pub(crate) fn announce(config: &ServerConfig) {
    logger::info!(
        "telividb listening on {} (grpc-web {}, reflection {})",
        config.addr,
        config.grpc_web,
        config.reflection
    );
    // Each macro returns a `LogBuilder` that emits when it drops — hence the
    // blocks: a bare match arm would make the arms' types the builder rather
    // than `()`.
    match config.otlp_addr {
        Some(addr) => {
            logger::info!("telemetry: exporting logs, traces and metrics to {addr}");
        }
        None => {
            logger::info!("telemetry: console only — pass --otlp <addr> to export");
        }
    }
    if let Some(path) = &config.mcap_path {
        logger::info!(
            "telemetry: recording MCAP for Foxglove at {}",
            path.display()
        );
    }
    logger::info!("telemetry: environment {}", config.environment);
    logger::info!("data directory: {}", config.data_dir.display());
    announce_device();
}

/// Say which device a GPU index would land on, at startup.
///
/// The one failure this design can have that no assertion catches: a build
/// with the GPU feature on, silently falling back to CPU because the backend
/// never initialised. It passes every correctness test — results are identical
/// — while delivering none of the speed. Reporting it once at startup makes
/// that visible before anyone benchmarks and wonders.
#[cfg(feature = "gpu")]
pub(crate) fn announce_device() {
    let name = telividb_index::adapters::Device::best().kind().as_str();
    match name {
        "cpu" => {
            logger::info!(
                "index device: cpu — no GPU backend initialised, search will be \
                 correct but unaccelerated"
            );
        }
        other => {
            logger::info!("index device: {other}");
        }
    }
    // Say where the ceiling came from, not just what it is. An *estimated*
    // budget on a discrete GPU is the case that overestimates — the operator
    // has to be able to tell that apart from a device-reported figure.
    logger::info!(
        "gpu budget: {:.1} GiB ({:.0}% of {}) — override with {}",
        telividb_index::adapters::gpu_budget_bytes() as f64 / (1024.0 * 1024.0 * 1024.0),
        telividb_index::adapters::DEFAULT_GPU_BUDGET_FRACTION * 100.0,
        telividb_index::adapters::budget_source().as_str(),
        telividb_index::adapters::BUDGET_ENV,
    );
}

/// Without the GPU feature there is no device to report.
#[cfg(not(feature = "gpu"))]
pub(crate) fn announce_device() {
    logger::info!("index device: cpu — built without the gpu feature");
}

/// Summarise what is currently resident.
///
/// Called on a signal rather than on a timer: a periodic dump would be noise
/// on an idle server and would not be there when someone actually needs it.
///
/// **Expect this to be empty today, and that is not a bug.** `PointsSvc` opens
/// a store per request and drops it as the handler returns, so nothing is
/// long-lived enough to still be registered at shutdown. It starts reporting
/// real rows once something is *held* — a cached store handle, or a
/// device-resident index or model, which is what the vector service and the
/// inference server introduce.
/// Names are redacted here even though the registry holds them raw — rule 28
/// governs what reaches a pipeline, and a store path carries its collection's
/// name, which may be a vault's.
pub(crate) fn announce_residency() {
    use telividb_telemetry::residency::{self, Location};

    let entries = residency::snapshot();
    if entries.is_empty() {
        return;
    }
    logger::info!(
        "resident: {} things, {:.1} MiB host, {:.1} MiB device",
        entries.len(),
        residency::total_bytes(Location::Host) as f64 / (1024.0 * 1024.0),
        residency::total_bytes(Location::Device) as f64 / (1024.0 * 1024.0),
    );
    for entry in entries {
        logger::debug!("resident item").with_data(&serde_json::json!({
            fields::RESIDENT_KIND: entry.kind.as_str(),
            fields::LOCATION: entry.location.as_str(),
            fields::RESOURCE: redact::resource_token(&entry.name),
            fields::RESIDENT_BYTES: entry.bytes,
        }));
    }
}
