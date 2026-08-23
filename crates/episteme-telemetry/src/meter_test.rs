use super::*;

#[test]
fn the_default_handle_records_nothing() {
    // A library crate constructing its own types must not start recording into
    // a pipeline nobody asked for, and a unit test must not need a runtime.
    let meter = Meter::default();
    assert!(!meter.is_enabled());
}

#[test]
fn a_disabled_handle_accepts_every_emission_without_panicking() {
    // Every call site emits unconditionally; the handle is what decides whether
    // it goes anywhere. If a disabled handle could panic, that decision would
    // have to be duplicated at each site.
    let meter = Meter::disabled();
    meter.counter("episteme_wal_bytes_total", 512.0);
    meter.histogram("episteme_search_duration_seconds", 0.25);
    meter.gauge("episteme_rows_live", 12.0);
}

#[test]
fn a_clone_shares_the_same_state() {
    let meter = Meter::disabled();
    let cloned = meter.clone();
    assert_eq!(meter.is_enabled(), cloned.is_enabled());
}

#[test]
fn debug_reveals_only_whether_recording_is_on() {
    // The recorder holds instrument caches and an MCAP writer. Invariant 28
    // keeps that out of a log.
    let rendered = format!("{:?}", Meter::disabled());
    assert_eq!(rendered, "Meter(false)");
}
