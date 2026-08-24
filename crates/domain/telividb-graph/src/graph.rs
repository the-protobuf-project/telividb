//! The in-memory property graph.
//!
//! Rehydrated from a [`GraphStore`] on collection load, never its own
//! persisted format (CLAUDE.md rule 47). A `petgraph::Graph` rather than a
//! `GraphMap`: two points can plausibly carry more than one edge type between
//! them (e.g. both `MENTIONS` and `CO_OCCURS`, ARCHITECTURE.md §5.2), and
//! `GraphMap` keys nodes by their weight, which allows at most one edge per
//! pair.

use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::time::Instant;
use telividb_core::{Edge, GraphStore, ResourceName, Result};
use telividb_telemetry::{fields, logger};

/// A directed, typed graph over point resource names.
///
/// Node weights are the resource names themselves; edge weights are the full
/// [`Edge`] record, so a traversal can read the edge type and weight without a
/// second lookup. The `index` map exists because `petgraph::Graph` addresses
/// nodes by an opaque [`NodeIndex`], not by the resource name a caller
/// actually has.
#[derive(Debug, Default)]
pub struct Graph {
    pub(crate) inner: DiGraph<ResourceName, Edge>,
    pub(crate) index: HashMap<ResourceName, NodeIndex>,
}

impl Graph {
    /// An empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Rebuild the whole graph from a store's edges, in one pass.
    ///
    /// This is the "on collection load" rehydration rule 47 describes: the
    /// store is scanned once, not consulted per traversal step.
    pub fn rehydrate(store: &dyn GraphStore) -> Result<Self> {
        let started = Instant::now();
        let mut graph = Self::new();
        for edge in store.iter_edges()? {
            graph.insert_edge(edge?);
        }

        // Worth a record on every collection load: this is the whole graph in
        // RAM (rule 47), so its size is the capacity ceiling anyone debugging
        // memory pressure needs, and a graph that came back empty is the
        // first symptom of edges never having been written.
        logger::info!("graph rehydrated").with_data(&serde_json::json!({
            fields::NODES: graph.node_count(),
            fields::EDGES: graph.edge_count(),
            fields::DURATION_SECONDS: started.elapsed().as_secs_f64(),
        }));
        Ok(graph)
    }

    /// Add one edge, creating either endpoint as a node if it is new.
    ///
    /// A node with no edges yet is not an error here — bulk import lands
    /// nodes and edges in separate passes (AGENT_START.md §7.6), so an edge
    /// may be the first thing this graph ever hears about one of its
    /// endpoints.
    pub fn insert_edge(&mut self, edge: Edge) {
        let src = self.node_index(edge.src.clone());
        let dst = self.node_index(edge.dst.clone());
        self.inner.add_edge(src, dst, edge);
    }

    /// The index for `name`, inserting a node for it if this is the first
    /// time the graph has seen it.
    fn node_index(&mut self, name: ResourceName) -> NodeIndex {
        if let Some(&index) = self.index.get(&name) {
            return index;
        }
        let index = self.inner.add_node(name.clone());
        self.index.insert(name, index);
        index
    }

    /// Number of distinct resources this graph knows about.
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Number of edges this graph holds, counting parallel edges separately.
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// Whether `name` appears as an endpoint of any edge.
    pub fn contains_node(&self, name: &ResourceName) -> bool {
        self.index.contains_key(name)
    }
}

#[cfg(test)]
#[path = "graph_test.rs"]
mod tests;
