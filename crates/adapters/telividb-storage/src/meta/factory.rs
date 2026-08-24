//! Choosing which concrete adapter backs a `GraphStore` or `PointStore`.
//!
//! Consumers of either port — `telividb_graph::Graph::rehydrate`, and the
//! document service's `PointsSvc` — already only ever see `dyn GraphStore` /
//! `dyn PointStore`; that genericity comes free from the ports living in
//! `telividb-core`. What is missing without this file is a single place that
//! *constructs* one: a caller reaching for `RedbGraphStore::open` or
//! `RedbPointStore::open` directly still has to know a concrete adapter
//! exists, which is the seam this closes for both.
//!
//! `redb` is the only backend today, for both. A `Postgres` or `Arango`
//! variant later is one new arm in the matching `open_*` function, never a
//! change to `telividb-core`, `telividb-graph`, or any caller already
//! matching on a config value by value rather than reaching past it.

use crate::error::Result;
use crate::meta::{RedbGraphStore, RedbPointStore};
use std::path::PathBuf;
use telividb_core::{GraphStore, PointStore};

/// Which concrete adapter backs a [`GraphStore`], and what it needs to open.
///
/// Selected at compile time from configuration, the same way an index kind
/// is (CLAUDE.md rule 22) — never a runtime plugin loader picking among
/// dynamically loaded backends.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum GraphStoreConfig {
    /// A `redb`-backed store at a local file path.
    Redb {
        /// Where the `redb` database file lives.
        path: PathBuf,
    },
}

/// Construct the adapter `config` selects, returned as the port it
/// implements rather than its concrete type.
///
/// This is the only function in the crate allowed to name a concrete
/// `GraphStore` adapter directly — everything downstream, including the
/// integration tests, goes through this and sees `Box<dyn GraphStore>`.
pub fn open_graph_store(config: &GraphStoreConfig) -> Result<Box<dyn GraphStore>> {
    match config {
        GraphStoreConfig::Redb { path } => Ok(Box::new(RedbGraphStore::open(path)?)),
    }
}

/// Which concrete adapter backs a [`PointStore`], and what it needs to open.
/// Same reasoning as [`GraphStoreConfig`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PointStoreConfig {
    /// A `redb`-backed store at a local file path.
    Redb {
        /// Where the `redb` database file lives.
        path: PathBuf,
    },
}

/// Construct the adapter `config` selects, returned as the port it
/// implements. The only function allowed to name a concrete `PointStore`
/// adapter directly — everything downstream sees `Box<dyn PointStore>`.
pub fn open_point_store(config: &PointStoreConfig) -> Result<Box<dyn PointStore>> {
    match config {
        PointStoreConfig::Redb { path } => Ok(Box::new(RedbPointStore::open(path)?)),
    }
}

#[cfg(test)]
#[path = "factory_test.rs"]
mod tests;
