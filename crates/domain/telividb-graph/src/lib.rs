//! Plan A1.1 — the property graph layered over the same points as the vector
//! store.
//!
//! Nodes are points that already exist; this crate only adds edges. The graph
//! itself is rehydrated in memory from a [`telividb_core::GraphStore`] on
//! collection load, not persisted in its own format — see CLAUDE.md rule 47.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod graph;
mod traversal;

pub use graph::Graph;
