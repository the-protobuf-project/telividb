//! What is currently resident, and how much it holds.
//!
//! One process-wide registry of every large thing alive — open stores, loaded
//! indexes, and (once the inference server lands) resident models. It answers
//! the question a model zoo actually needs answered: *what is on the GPU right
//! now, and which collection or model does it belong to.*
//!
//! # Why the accounting is shared rather than per crate
//!
//! A GPU budget that only counted vector indexes would be wrong the moment a
//! model is resident beside them: both occupy the same device, so both must
//! draw on the same ceiling. Keeping one registry here — in the crate every
//! emitting crate already depends on — is what lets `telividb-index`'s budget
//! see memory that `telividb-embed` allocated, without either crate depending
//! on the other.
//!
//! # Names are stored raw and redacted at emission
//!
//! This is the one thing in this module worth getting right. A local operator
//! listing what is resident needs *real* names to act on — the `ollama ps`
//! equivalent is useless if every row says `r_8f3a…`. Rule 28 governs what
//! reaches a telemetry *pipeline*, not what a process may hold in memory, so:
//!
//! - [`snapshot`] returns real names, for in-process use and operator tooling.
//! - Anything emitting a registry entry passes the name through
//!   [`redact::resource_token`](crate::redact::resource_token) or
//!   [`redact::collection_label`](crate::redact::collection_label) first.
//!
//! Emitting a raw entry name would disclose vault names, which is exactly the
//! disclosure rule 28 exists to prevent.

use std::sync::{Mutex, OnceLock};

/// What kind of thing is holding memory.
///
/// A closed set, so it is safe as a metric label ([`fields::LABEL_SAFE`](crate::fields::LABEL_SAFE)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentKind {
    /// A point store's backing file.
    PointStore,
    /// A graph store's backing file.
    GraphStore,
    /// A vector index, which may be device-resident.
    VectorIndex,
    /// A loaded inference model.
    Model,
}

impl ResidentKind {
    /// The name used in telemetry and in operator output.
    pub fn as_str(self) -> &'static str {
        match self {
            ResidentKind::PointStore => "point-store",
            ResidentKind::GraphStore => "graph-store",
            ResidentKind::VectorIndex => "vector-index",
            ResidentKind::Model => "model",
        }
    }
}

/// Where the memory actually sits.
///
/// The distinction the GPU budget turns on: only [`Location::Device`] competes
/// for VRAM, while host allocations are bounded by ordinary system memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Location {
    /// System memory — a mapped file, a host-side buffer, page cache. Bounded
    /// by ordinary RAM, and never counted against the GPU ceiling.
    Host,
    /// GPU memory — or, on unified-memory hardware, the GPU-addressable pool.
    Device,
}

impl Location {
    /// The name used in telemetry and in operator output.
    pub fn as_str(self) -> &'static str {
        match self {
            Location::Host => "host",
            Location::Device => "device",
        }
    }
}

/// One resident thing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// What kind of thing this is.
    pub kind: ResidentKind,
    /// Where its memory sits.
    pub location: Location,
    /// **Real** name — a collection, a field, a model file. Redact before
    /// emitting; see the module docs.
    pub name: String,
    /// Bytes it holds.
    pub bytes: usize,
    /// Distinguishes entries that would otherwise be identical.
    id: u64,
}

/// The registry. A `Mutex<Vec<_>>` rather than anything cleverer because
/// registration happens on load and drop, not on the query path — this is
/// never contended by a search.
fn registry() -> &'static Mutex<Vec<Entry>> {
    static REGISTRY: OnceLock<Mutex<Vec<Entry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// Monotonic id source, so two same-named entries stay distinguishable.
fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// A registration, removed when dropped.
///
/// Held by the thing it describes, so release is tied to that thing's own
/// lifetime rather than to a matching call someone has to remember.
#[derive(Debug)]
pub struct Handle {
    id: u64,
    bytes: usize,
}

impl Handle {
    /// Bytes this registration accounts for.
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if let Ok(mut entries) = registry().lock() {
            entries.retain(|e| e.id != self.id);
        }
    }
}

/// Record that `name` is resident, holding `bytes`.
pub fn register(
    kind: ResidentKind,
    location: Location,
    name: impl Into<String>,
    bytes: usize,
) -> Handle {
    let id = next_id();
    if let Ok(mut entries) = registry().lock() {
        entries.push(Entry {
            kind,
            location,
            name: name.into(),
            bytes,
            id,
        });
    }
    Handle { id, bytes }
}

/// Everything currently resident, with **real** names. Redact before emitting.
pub fn snapshot() -> Vec<Entry> {
    registry().lock().map(|e| e.clone()).unwrap_or_default()
}

/// Total bytes held in one location.
///
/// This is what the GPU budget checks, which is why a model and an index
/// resident together are correctly seen as competing for the same memory.
pub fn total_bytes(location: Location) -> usize {
    registry()
        .lock()
        .map(|entries| {
            entries
                .iter()
                .filter(|e| e.location == location)
                .map(|e| e.bytes)
                .sum()
        })
        .unwrap_or(0)
}

/// How many things of one kind are resident.
pub fn count(kind: ResidentKind) -> usize {
    registry()
        .lock()
        .map(|entries| entries.iter().filter(|e| e.kind == kind).count())
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "residency_test.rs"]
mod tests;
