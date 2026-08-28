use super::*;
use crate::adapters::gpu::test_support::exclusive;

/// A size small enough that any plausible budget admits it, so tests about
/// accounting never accidentally test the ceiling.
const SMALL: usize = 1024;

#[test]
fn a_reservation_is_released_on_drop() {
    let _guard = exclusive();
    let before = resident_bytes();
    {
        let held = reserve("test", SMALL).unwrap();
        assert_eq!(held.bytes(), SMALL);
        assert_eq!(resident_bytes(), before + SMALL);
    }
    assert_eq!(
        resident_bytes(),
        before,
        "dropping an index must return its bytes to the budget"
    );
}

#[test]
fn reservations_accumulate_across_live_indexes() {
    let _guard = exclusive();
    // The case that actually killed the process: a rebuild holds the old
    // corpus and the new one at once, so the ceiling has to see both.
    let before = resident_bytes();
    let _a = reserve("test", SMALL).unwrap();
    let _b = reserve("test", SMALL).unwrap();
    assert_eq!(resident_bytes(), before + 2 * SMALL);
}

#[test]
fn an_oversized_request_is_refused_rather_than_attempted() {
    let _guard = exclusive();
    // Refusing is the whole point: an over-large allocation aborts the process on
    // upload, so a `Result` here is the only recoverable form the failure has.
    let err = reserve("test", limit_bytes() + 1).unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains("budget"),
        "the error should explain the ceiling: {message}"
    );
    assert!(
        message.contains(BUDGET_ENV),
        "the error should name the override: {message}"
    );
}

#[test]
fn a_refused_reservation_leaves_the_counter_untouched() {
    let _guard = exclusive();
    let before = resident_bytes();
    let _ = reserve("test", limit_bytes() + 1);
    assert_eq!(
        resident_bytes(),
        before,
        "a refusal must not consume budget"
    );
}

#[test]
fn a_resident_model_consumes_the_same_ceiling_an_index_does() {
    // The whole point of routing through the shared registry: once the
    // inference server lands, a model on the GPU competes with indexes for the
    // same memory. A per-crate counter could not see it.
    let _guard = exclusive();
    use telividb_telemetry::residency::{self, Location, ResidentKind};

    let before = resident_bytes();
    let _model = residency::register(
        ResidentKind::Model,
        Location::Device,
        "nomic-embed-text-v1.5.gguf",
        SMALL,
    );
    assert_eq!(
        resident_bytes(),
        before + SMALL,
        "a model must count against the index budget"
    );
}

#[test]
fn a_host_resident_store_does_not_consume_the_device_budget() {
    // The converse: a point store's file is host memory and must not shrink
    // the ceiling an index competes for.
    let _guard = exclusive();
    use telividb_telemetry::residency::{self, Location, ResidentKind};

    let before = resident_bytes();
    let _store = residency::register(
        ResidentKind::PointStore,
        Location::Host,
        "collections/media",
        SMALL,
    );
    assert_eq!(resident_bytes(), before, "host memory is not device memory");
}

#[test]
fn the_budget_reports_where_it_came_from() {
    // An operator has to be able to tell a device measurement from a guess:
    // an estimated ceiling on a discrete GPU is the case that overestimates.
    let _guard = exclusive();
    assert!(matches!(
        budget_source(),
        BudgetSource::Configured | BudgetSource::DeviceReported | BudgetSource::Estimated
    ));
}

#[test]
fn the_budget_is_a_fraction_of_something_real() {
    let _guard = exclusive();
    // Not asserting an exact figure — it is host-dependent — but a budget of
    // zero would silently refuse every index, and an enormous one would mean
    // detection failed open.
    let limit = limit_bytes();
    assert!(limit > 0, "a zero budget refuses everything");
    assert!(
        limit >= 64 * 1024 * 1024,
        "budget {limit} is implausibly small; detection likely failed"
    );
}
