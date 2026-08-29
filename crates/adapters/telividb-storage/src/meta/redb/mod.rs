//! The `redb` metadata stores.
//!
//! `redb` is the one mutable store in this crate: segments are immutable and
//! `mmap`'d, so everything that has to change in place — a catalogue entry, an
//! edge, a tenancy record — lives here instead.
//!
//! One module per resource group. Each owns its table definitions, its record
//! encodings, and the tests for both.

pub mod collection;
pub mod cursor;
pub mod graph;
pub mod point;
pub mod tenancy;

pub use collection::store::RedbCollectionStore;
pub use graph::store::RedbGraphStore;
pub use point::store::RedbPointStore;
pub use tenancy::store::RedbTenancyStore;
