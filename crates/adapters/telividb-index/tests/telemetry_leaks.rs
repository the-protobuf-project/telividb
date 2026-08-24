//! Telemetry must not leak what the security model protects.
//!
//! Logs and traces routinely land in systems with weaker access control than
//! the database, are retained longer, and are read by people who were never
//! granted anything. A pipeline that records query vectors hands out precisely
//! what separating `search` from `read_vector` exists to prevent — and a vector
//! can be inverted back toward its source text.
//!
//! So this is a regression test, not documentation: run a search through the
//! real pipeline, recording to MCAP, then read the recording back and assert
//! the vectors are not in it.
//!
//! Recording rather than capturing a subscriber is deliberate. MCAP is a
//! destination an operator actually configures, and it is a *file* — the exact
//! shape of artefact that gets copied into a ticket. Asserting against the
//! bytes that land on disk is the closest available test to the thing that
//! would do the leaking.

use std::sync::OnceLock;
use telividb_core::{Dim, Metric};
use telividb_index::{FlatIndex, VectorIndex, adapters::MemoryStore};
use telividb_telemetry::{Telemetry, TelemetryConfig};

/// Values chosen so each renders as a distinctive substring — if any component
/// reaches the recording, a plain byte search will find it.
const QUERY: [f32; 4] = [0.918_273, -0.284_951, 0.635_412, -0.771_236];
const STORED: [f32; 4] = [0.412_589, 0.883_167, -0.194_726, 0.557_038];

/// The dimension both vectors share, which *is* safe to emit.
const DIM: u32 = 4;

/// The recording, made exactly once for the whole test binary.
///
/// The stack installs a *global* logger, so a second `install` in this process
/// would not take effect and every assertion reading its file would be looking
/// at an empty recording — passing for the wrong reason. Recording once and
/// sharing the bytes is what keeps each test honest.
static RECORDING: OnceLock<Vec<u8>> = OnceLock::new();

/// Run one search with the pipeline recording, and return the recording.
fn recorded_search() -> &'static [u8] {
    RECORDING.get_or_init(record_once)
}

/// Install the pipeline, search once, and read back what was written.
fn record_once() -> Vec<u8> {
    let dir = tempfile::tempdir().expect("temp dir");
    let mcap = dir.path().join("telemetry.mcap");

    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let bytes = runtime.block_on(async {
        let telemetry = Telemetry::install(TelemetryConfig {
            service: "telividb-leak-test".to_owned(),
            mcap_path: Some(mcap.clone()),
            config_path: Some(hermetic_config(dir.path())),
            ..TelemetryConfig::default()
        })
        .expect("telemetry installs");

        let mut store = MemoryStore::new(Dim::new(DIM).unwrap(), Metric::Dot);
        store.push(&STORED).unwrap();
        let index = FlatIndex::new().with_meter(telemetry.meter());
        let hits = index.search(&store, &QUERY, 1, None).unwrap();
        assert_eq!(hits.len(), 1, "the search itself must still work");

        // Flush is best-effort here: with no collector configured the stack's
        // meter has nothing to force-flush and reports that as an error, which
        // says nothing about the MCAP file. Dropping the pipeline closes the
        // writer, and that is what makes the file readable rather than a
        // truncated stream.
        let _ = telemetry.flush();
        drop(telemetry);
        std::fs::read(&mcap).expect("recording exists")
    });

    assert!(
        !bytes.is_empty(),
        "the recording is empty, so every assertion below would pass vacuously"
    );
    bytes
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

/// The recording, decoded so that message payloads are actually searchable.
fn recorded_text() -> String {
    decoded(recorded_search())
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

/// The distinctive fractional digits of a component.
///
/// Matching on these rather than the whole rendering means a coincidental
/// `0.9` elsewhere in the file cannot mask a real leak, and a value written
/// with a different precision or sign still trips the check.
fn needle(value: f32) -> String {
    format!("{:.6}", value.abs())
        .trim_start_matches("0.")
        .to_owned()
}

#[test]
fn no_query_vector_component_is_recorded() {
    let out = recorded_text();
    for value in QUERY {
        assert!(
            !out.contains(&needle(value)),
            "query component {value} leaked into the recording"
        );
    }
}

#[test]
fn no_stored_vector_component_is_recorded() {
    let out = recorded_text();
    for value in STORED {
        assert!(
            !out.contains(&needle(value)),
            "stored component {value} leaked into the recording"
        );
    }
}

#[test]
fn the_shape_of_the_query_is_still_recorded() {
    // Redaction that emits nothing is easy and useless. Dimension is what an
    // operator actually needs to debug a mismatch, and it discloses nothing.
    //
    // This is also what stops the tests above passing for the wrong reason: if
    // the search stopped emitting entirely, this fails.
    let out = recorded_text();
    assert!(
        out.contains(&DIM.to_string()),
        "the query shape is missing, so the leak checks prove nothing"
    );
}

#[test]
fn operational_counts_are_still_recorded() {
    let out = recorded_text();
    for expected in ["candidates_visited", "results_returned"] {
        assert!(
            out.contains(expected),
            "missing {expected} in the recording"
        );
    }
}
