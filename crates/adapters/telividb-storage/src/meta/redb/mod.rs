//! The `redb` metadata stores.
//!
//! `redb` is the one mutable store in this crate: segments are immutable and
//! `mmap`'d, so everything that has to change in place — a catalogue entry, an
//! edge, a tenancy record — lives here instead.
//!
//! One module per resource. Each owns its table definition, its record
//! encoding, and the tests for both.

pub mod collection;
pub mod cursor;
pub mod graph;
pub mod organization;
pub mod point;

pub use collection::store::RedbCollectionStore;
pub use graph::store::RedbGraphStore;
pub use organization::store::RedbTenancyStore;
pub use point::store::RedbPointStore;
