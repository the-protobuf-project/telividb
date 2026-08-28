//! One lock, shared by every test that touches device residency.
//!
//! The residency registry is process-wide and the harness runs tests in
//! parallel, so a test asserting on `resident_bytes()` races any *other* test
//! that builds or drops an index. The budget tests read the total twice — once
//! as a baseline, once as an assertion — and a reservation landing between
//! those two reads makes the delta wrong.
//!
//! The index-building harness lives here for the same reason: every test that
//! builds one must hold the lock, so the two belong together rather than in
//! whichever test file happened to need them first.
//!
//! This is why the lock lives here rather than inside `budget_test.rs`, where
//! it started: serialising the budget tests against each other was enough only
//! while nothing else reserved. Building an index now does, so the lock has to
//! cover both.

use super::GpuFlatIndex;
use crate::adapters::MemoryStore;
use crate::domain::Candidate;
use crate::ports::VectorIndex;
use telividb_compute::DeviceKind;
use telividb_core::{Ordinal, Result};

/// Held for the body of any test that reserves, releases, or measures device
/// residency.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Take the shared lock.
///
/// Recovers from poisoning rather than propagating it: a panicking test has
/// already failed and reported, and turning that into a cascade of unrelated
/// failures in every test after it hides which one actually broke.
pub(super) fn exclusive() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// An index and the store it was built from.
///
/// Kept together because the port takes both. This index never reads the store
/// back — it owns a device-resident copy — but a test should exercise the real
/// signature rather than a convenient one, so the store stays alive and gets
/// passed.
pub(super) struct Built {
    index: GpuFlatIndex,
    pub(super) store: MemoryStore,
    /// Held for the test's lifetime: building an index reserves device
    /// residency, and the budget tests measure that total. See
    /// [`crate::adapters::gpu::test_support`].
    _serial: std::sync::MutexGuard<'static, ()>,
}

impl Built {
    /// Build on the CPU backend, which every machine has.
    ///
    /// Named rather than `best()`: results are identical on every backend, so
    /// a unit test gains nothing from a GPU and would otherwise be skipped on
    /// CI machines that have none. `gpu_recall.rs` is where a real device is
    /// exercised.
    pub(super) fn on_cpu(store: MemoryStore) -> Self {
        let _serial = exclusive();
        let index = GpuFlatIndex::build_on(&store, DeviceKind::Cpu).unwrap();
        Self {
            index,
            store,
            _serial,
        }
    }

    pub(super) fn search(&self, query: &[f32], k: usize) -> Result<Vec<Candidate>> {
        self.index.search(&self.store, query, k, None)
    }

    pub(super) fn search_visible(
        &self,
        query: &[f32],
        k: usize,
        allowed: &dyn Fn(Ordinal) -> bool,
    ) -> Result<Vec<Candidate>> {
        self.index.search(&self.store, query, k, Some(allowed))
    }
}

/// An index on the CPU backend, with the guard that must outlive it.
///
/// CPU rather than `best()`: results are identical on every backend, so a unit
/// test gains nothing from a GPU and would otherwise be skipped wherever CI has
/// none. `tests/gpu_recall.rs` is where a real device is exercised.
///
/// The batched tests need the index and the store separately — the port takes
/// both — so this returns the pieces rather than the [`Built`] pairing.
pub(super) fn index_of(store: &MemoryStore) -> (GpuFlatIndex, std::sync::MutexGuard<'static, ()>) {
    let serial = exclusive();
    (
        GpuFlatIndex::build_on(store, DeviceKind::Cpu).unwrap(),
        serial,
    )
}
