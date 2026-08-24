//! Helper functions for batched parallel construction.

use crate::adapters::hnsw::build::prune;
use crate::adapters::hnsw::graph::Graph;
use crate::adapters::hnsw::params::HnswParams;
use crate::adapters::hnsw::scored::Scored;
use crate::adapters::hnsw::search::{distance_to, greedy_descend, search_layer};
use crate::adapters::hnsw::select::select_neighbours;
use crate::adapters::hnsw::visited::VisitedSet;
use episteme_core::{Metric, Ordinal, VectorStore};

/// The read-only half of an insert: candidates for each layer this node joins.
///
/// Touches nothing mutable, so any number of these run concurrently against one
/// graph snapshot.
#[allow(clippy::too_many_arguments)]
pub fn find_candidates(
    graph: &Graph,
    store: &dyn VectorStore,
    metric: Metric,
    params: &HnswParams,
    vector: &[f32],
    level: usize,
    visited: &mut VisitedSet,
) -> Vec<Vec<Scored>> {
    let Some(entry) = graph.entry() else {
        return Vec::new();
    };
    let Some(entry_distance) = distance_to(store, metric, vector, entry) else {
        return Vec::new();
    };

    let previous_max = graph.max_level();
    let mut cursor = Scored::new(entry_distance, entry);
    for layer in ((level + 1)..=previous_max).rev() {
        cursor = greedy_descend(graph, store, metric, vector, cursor, layer);
    }

    // Highest layer first, matching the order `apply` consumes them in.
    let mut per_layer = Vec::new();
    for layer in (0..=level.min(previous_max)).rev() {
        let candidates = search_layer(
            graph,
            store,
            metric,
            vector,
            cursor,
            params.ef_construction,
            layer,
            None,
            visited,
        );
        if let Some(best) = candidates.first() {
            cursor = *best;
        }
        per_layer.push(candidates);
    }
    per_layer
}

/// The mutating half of an insert: link the node using candidates already found.
#[allow(clippy::too_many_arguments)]
pub fn apply(
    graph: &mut Graph,
    store: &dyn VectorStore,
    metric: Metric,
    params: &HnswParams,
    level: usize,
    per_layer: Vec<Vec<Scored>>,
    _visited: &mut VisitedSet,
) {
    let previous_max = graph.max_level();
    let node = graph.push_node(level);

    for (offset, candidates) in per_layer.into_iter().enumerate() {
        if candidates.is_empty() {
            continue;
        }
        // `find_candidates` walked layers downward from the node's level.
        let layer = level.min(previous_max).saturating_sub(offset);

        let budget = params.max_neighbours(layer);
        let chosen = select_neighbours(store, metric, &candidates, budget);
        graph.set_neighbours(node, layer, chosen.clone());

        for neighbour in chosen {
            if graph.try_connect(neighbour, layer, node, budget) {
                prune(graph, store, metric, neighbour, layer, budget);
            }
        }
    }
}
