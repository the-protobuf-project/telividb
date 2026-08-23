//! The pipeline actually carries what the workspace emits.
//!
//! This exists because the failure it guards against is invisible at every call
//! site. `metrics::counter!` with no recorder installed, and `tracing::info!`
//! with no subscriber, both compile, run, and record nothing — so the entire
//! instrumentation of the database was disconnected from its pipeline while
//! every emission site still read as correct.
//!
//! Asserting against an MCAP recording rather than a captured subscriber is
//! deliberate: MCAP is a destination an operator configures, and it is a file,
//! so the assertion is against bytes that really landed somewhere.

use episteme_telemetry::{Telemetry, TelemetryConfig, catalogue, logger, metrics_names};
use std::sync::OnceLock;

/// The recording, made exactly once for the whole test binary.
///
/// The stack installs a *global* logger, so a second `install` in this process
/// would not take effect and every assertion reading its file would be looking
/// at an empty recording — passing for the wrong reason.
static RECORDING: OnceLock<Vec<u8>> = OnceLock::new();

/// A value distinctive enough that finding it in the file proves it came from
/// here rather than from the stack's own startup chatter.
const MARKER_BYTES: f64 = 987_654.0;

fn recording() -> &'static [u8] {
    RECORDING.get_or_init(|| {
        let dir = tempfile::tempdir().expect("temp dir");
        let mcap = dir.path().join("pipeline.mcap");
        let runtime = tokio::runtime::Runtime::new().expect("runtime");

        let bytes = runtime.block_on(async {
            let telemetry = Telemetry::install(TelemetryConfig {
                service: "episteme-pipeline-test".to_owned(),
                mcap_path: Some(mcap.clone()),
                config_path: Some(hermetic_config(dir.path())),
                ..TelemetryConfig::default()
            })
            .expect("telemetry installs");

            let meter = telemetry.meter();
            assert!(
                meter.is_enabled(),
                "an installed pipeline must hand out a recording meter"
            );
            meter.counter(metrics_names::WAL_BYTES, MARKER_BYTES);
            meter.gauge(metrics_names::ROWS_LIVE, 42.0);
            meter.histogram(metrics_names::SEARCH_DURATION, 0.125);
            logger::info!("pipeline test log record");

            let _ = telemetry.flush();
            drop(telemetry);
            std::fs::read(&mcap).expect("recording exists")
        });

        assert!(
            !bytes.is_empty(),
            "the recording is empty, so every assertion below would pass vacuously"
        );
        bytes
    })
}

/// Write a stack config that keeps this test off the network.
///
/// With no config file discoverable — and a test binary's working directory is
/// never the repository root — the stack falls back to defaults, where
/// `[telemetry] enabled` is **true**. An OTLP exporter then starts and reports
/// a refused connection on every batch. Writing one explicitly is what keeps
/// the test hermetic.
fn hermetic_config(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("telemetry.toml");
    std::fs::write(
        &path,
        "[telemetry]\nenabled = false\n\n[telemetry.otlp]\nenabled = false\n",
    )
    .expect("write config");
    path
}

fn recorded_text() -> String {
    decoded(recording())
}

/// Everything the recording contains, decoded.
///
/// MCAP compresses message payloads (zstd by default), so a substring search
/// over the raw file finds only schema and channel metadata — the *names* of
/// things, never the values. Any assertion about what was or was not recorded
/// has to decompress first, or it passes regardless of what the pipeline did.
fn decoded(bytes: &[u8]) -> String {
    let mut out = String::new();
    let stream = mcap::MessageStream::new(bytes).expect("the recording parses as MCAP");
    let mut messages = 0usize;
    for message in stream {
        let message = message.expect("each record parses");
        out.push_str(&message.channel.topic);
        out.push('\n');
        if let Some(schema) = &message.channel.schema {
            out.push_str(&schema.name);
            out.push('\n');
            out.push_str(&String::from_utf8_lossy(&schema.data));
            out.push('\n');
        }
        out.push_str(&String::from_utf8_lossy(&message.data));
        out.push('\n');
        messages += 1;
    }
    assert!(
        messages > 0,
        "the recording holds no messages, so every assertion would pass vacuously"
    );
    out
}

#[test]
fn a_counter_reaches_the_pipeline() {
    // The regression: with no recorder installed anywhere in the process, this
    // emission went nowhere and nothing said so.
    let out = recorded_text();
    assert!(
        out.contains(metrics_names::WAL_BYTES),
        "{} never reached the recording",
        metrics_names::WAL_BYTES
    );
}

#[test]
fn a_gauge_reaches_the_pipeline() {
    let out = recorded_text();
    assert!(
        out.contains(metrics_names::ROWS_LIVE),
        "{} never reached the recording",
        metrics_names::ROWS_LIVE
    );
}

#[test]
fn a_histogram_reaches_the_pipeline() {
    let out = recorded_text();
    assert!(
        out.contains(metrics_names::SEARCH_DURATION),
        "{} never reached the recording",
        metrics_names::SEARCH_DURATION
    );
}

#[test]
fn a_log_record_reaches_the_pipeline() {
    // The other half of the same failure: the stack wires a `tracing`
    // subscriber only alongside an OTLP tracer, so anything emitted through
    // `tracing` rather than `logger::` is silent in exactly this configuration.
    let out = recorded_text();
    assert!(
        out.contains("pipeline test log record"),
        "the log record never reached the recording"
    );
}

#[test]
fn the_recorded_value_is_the_one_emitted() {
    // Name-only assertions would pass against a pipeline that registered the
    // instrument and dropped every sample.
    let out = recorded_text();
    assert!(
        out.contains("987654"),
        "the counter's value never reached the recording"
    );
}

#[test]
fn a_disabled_pipeline_still_accepts_every_emission() {
    // What a library crate, a benchmark and a unit test all get. If this could
    // panic, every emission site would need a guard around it.
    let meter = episteme_telemetry::Meter::disabled();
    for (name, kind, _) in catalogue::ALL {
        match kind {
            episteme_telemetry::MetricType::Counter => meter.counter(name, 1.0),
            episteme_telemetry::MetricType::Gauge => meter.gauge(name, 1.0),
            episteme_telemetry::MetricType::Histogram => meter.histogram(name, 1.0),
        }
    }
    assert!(!meter.is_enabled());
}
