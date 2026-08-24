use super::*;

/// The registry is process-wide and the harness runs tests in parallel, so a
/// delta assertion in one test would race another test's registration. Every
/// test here takes this lock, which makes the deltas deterministic rather than
/// usually-right.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Take the lock, ignoring poisoning: a panicking test has already failed and
/// should not cascade into every other test in the file.
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}
#[test]
fn a_registration_is_visible_and_released_on_drop() {
    let _guard = serial();
    let before = total_bytes(Location::Device);
    {
        let held = register(ResidentKind::VectorIndex, Location::Device, "media", 4096);
        assert_eq!(held.bytes(), 4096);
        assert_eq!(total_bytes(Location::Device), before + 4096);
    }
    assert_eq!(
        total_bytes(Location::Device),
        before,
        "dropping a handle must return its bytes"
    );
}

#[test]
fn host_and_device_are_accounted_separately() {
    let _guard = serial();
    // The distinction the GPU budget turns on: a point store on disk must not
    // consume the device ceiling an index competes for.
    let device_before = total_bytes(Location::Device);
    let host_before = total_bytes(Location::Host);

    let _store = register(ResidentKind::PointStore, Location::Host, "media", 1000);
    assert_eq!(total_bytes(Location::Device), device_before);
    assert_eq!(total_bytes(Location::Host), host_before + 1000);
}

#[test]
fn a_model_and_an_index_share_one_device_total() {
    let _guard = serial();
    // The whole reason this registry is shared rather than per crate: a
    // resident model competes with an index for the same memory.
    let before = total_bytes(Location::Device);
    let _index = register(ResidentKind::VectorIndex, Location::Device, "media", 500);
    let _model = register(ResidentKind::Model, Location::Device, "nomic.gguf", 700);
    assert_eq!(total_bytes(Location::Device), before + 1200);
}

#[test]
fn entries_with_the_same_name_stay_distinct() {
    let _guard = serial();
    // A rebuild holds the old index and the new one, both named for the same
    // field. Dropping one must not remove the other.
    let before = total_bytes(Location::Device);
    let first = register(ResidentKind::VectorIndex, Location::Device, "text_bge", 100);
    let _second = register(ResidentKind::VectorIndex, Location::Device, "text_bge", 100);
    assert_eq!(total_bytes(Location::Device), before + 200);

    drop(first);
    assert_eq!(
        total_bytes(Location::Device),
        before + 100,
        "dropping one of two same-named entries must leave the other"
    );
}

#[test]
fn a_snapshot_carries_the_real_name() {
    let _guard = serial();
    // Deliberate: operator tooling needs actionable names. Redaction happens
    // at emission, not here — see the module docs.
    let _held = register(
        ResidentKind::GraphStore,
        Location::Host,
        "collections/finance",
        64,
    );
    let found = snapshot()
        .into_iter()
        .any(|e| e.name == "collections/finance" && e.kind == ResidentKind::GraphStore);
    assert!(found, "snapshot should expose the unredacted name");
}

#[test]
fn count_is_per_kind() {
    let _guard = serial();
    let before = count(ResidentKind::Model);
    let _a = register(ResidentKind::Model, Location::Device, "a.gguf", 1);
    let _b = register(ResidentKind::Model, Location::Device, "b.gguf", 1);
    assert_eq!(count(ResidentKind::Model), before + 2);
}

#[test]
fn every_kind_and_location_names_itself() {
    let _guard = serial();
    // These strings become metric labels, so a missing arm would surface as an
    // unlabelled series rather than a compile error.
    for kind in [
        ResidentKind::PointStore,
        ResidentKind::GraphStore,
        ResidentKind::VectorIndex,
        ResidentKind::Model,
    ] {
        assert!(!kind.as_str().is_empty());
    }
    assert_eq!(Location::Host.as_str(), "host");
    assert_eq!(Location::Device.as_str(), "device");
}
