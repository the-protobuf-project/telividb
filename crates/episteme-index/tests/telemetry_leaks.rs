//! Telemetry must not leak what the security model protects.
//!
//! Logs and traces routinely land in systems with weaker access control than
//! the database, are retained longer, and are read by people who were never
//! granted anything. A pipeline that records query vectors hands out precisely
//! what separating `search` from `read_vector` exists to prevent — and a vector
//! can be inverted back toward its source text.
//!
//! So this is a regression test, not documentation: capture everything emitted
//! during a search and assert the vector is not in it.

use episteme_core::{Dim, Metric};
use episteme_index::{FlatIndex, VectorIndex, adapters::MemoryStore};
use std::io;
use std::sync::{Arc, Mutex};

/// A writer that keeps everything written to it.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Capture {
    fn contents(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

impl io::Write for Capture {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
    type Writer = Self;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Values chosen so each renders as a distinctive substring — if any component
/// reaches the output, a plain text search will find it.
const QUERY: [f32; 4] = [0.918_273, -0.284_951, 0.635_412, -0.771_236];
const STORED: [f32; 4] = [0.412_589, 0.883_167, -0.194_726, 0.557_038];

fn search_capturing_telemetry() -> String {
    let capture = Capture::default();

    let subscriber = tracing_subscriber::fmt()
        .with_writer(capture.clone())
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .finish();

    tracing::subscriber::with_default(subscriber, || {
        let mut store = MemoryStore::new(Dim::new(4).unwrap(), Metric::Dot);
        store.push(&STORED).unwrap();
        let hits = FlatIndex.search(&store, &QUERY, 1, None).unwrap();
        assert_eq!(hits.len(), 1, "the search itself must still work");
    });

    capture.contents()
}

#[test]
fn no_query_vector_component_is_emitted() {
    let out = search_capturing_telemetry();
    for value in QUERY {
        // Match on the distinctive fractional digits, so a coincidental "0.9"
        // elsewhere in the output does not mask a real leak.
        let needle = format!("{:.6}", value.abs())
            .trim_start_matches("0.")
            .to_owned();
        assert!(
            !out.contains(&needle),
            "query component {value} leaked into telemetry:\n{out}"
        );
    }
}

#[test]
fn no_stored_vector_component_is_emitted() {
    let out = search_capturing_telemetry();
    for value in STORED {
        let needle = format!("{:.6}", value.abs())
            .trim_start_matches("0.")
            .to_owned();
        assert!(
            !out.contains(&needle),
            "stored component {value} leaked into telemetry:\n{out}"
        );
    }
}

#[test]
fn the_shape_of_the_query_is_still_reported() {
    // Redaction that emits nothing is easy and useless. Dimension is what an
    // operator actually needs to debug a mismatch, and it discloses nothing.
    let out = search_capturing_telemetry();
    assert!(out.contains("dim=4"), "query shape missing:\n{out}");
}

#[test]
fn operational_counters_are_still_reported() {
    let out = search_capturing_telemetry();
    assert!(
        out.contains("candidates_visited"),
        "missing scan count:\n{out}"
    );
    assert!(
        out.contains("results_returned"),
        "missing result count:\n{out}"
    );
}
