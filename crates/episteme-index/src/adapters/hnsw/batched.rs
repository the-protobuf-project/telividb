//! Batched parallel construction.
//!
//! Kept apart from the sequential insert path because it answers a different
//! question: not "how is a node linked" but "how is that work divided without
//! making the result depend on scheduling".
//!
//! Every node in a batch searches the same read-only snapshot concurrently;
//! their connections are then applied in row order on one thread. That ordering
//! is what makes a batched build reproducible.

use super::build::prune;
use super::graph::Graph;
use super::params::HnswParams;
use super::scored::Scored;
use super::search::{distance_to, greedy_descend, search_layer};
use super::select::select_neighbours;
use super::visited::VisitedSet;
use episteme_core::{Metric, Ordinal, VectorStore};

/// Insert in batches: search concurrently, apply in row order.
///
/// The two halves are deliberately separate. Searching is read-only, so every
/// node in a batch can do it at once against the same snapshot. Applying mutates
/// the graph, so it happens on one thread in a fixed order — which is what makes
/// the result independent of thread count and scheduling.
pub(super) fn build_batched(
    store: &dyn VectorStore,
    params: &HnswParams,
    levels: &[usize],
    metric: Metric,
) -> Graph {
    use rayon::prelude::*;

    let mut graph = Graph::new();
    let mut visited = VisitedSet::with_capacity(store.len());
    let rows = store.len();

    for start in (0..rows).step_by(params.batch_size) {
        let end = (start + params.batch_size).min(rows);

        // `map_init` builds the visited set once per worker thread rather than
        // once per node — the same allocation trap that made the sequential
        // build quadratic before.
        let found: Vec<Option<Vec<Vec<Scored>>>> = (start..end)
            .into_par_iter()
            .map_init(
                || VisitedSet::with_capacity(rows),
                |scratch, row| {
                    let vector = store.get(Ordinal::from_row(row as u32))?;
                    Some(find_candidates(
                        &graph,
                        store,
                        metric,
                        params,
                        vector,
                        levels[row],
                        scratch,
                    ))
                },
            )
            .collect();

        for (offset, candidates) in found.into_iter().enumerate() {
            let row = start + offset;
            match (candidates, store.get(Ordinal::from_row(row as u32))) {
                (Some(per_layer), Some(_)) => {
                    apply(
                        &mut graph,
                        store,
                        metric,
                        params,
                        levels[row],
                        per_layer,
                        &mut visited,
                    );
                }
                // Absent for this field: it still occupies an ordinal so stride
                // holds, but it is not a node in the graph.
                _ => {
                    graph.push_node(0);
                }
            }
        }
    }
    graph
}

/// The read-only half of an insert: candidates for each layer this node joins.
///
/// Touches nothing mutable, so any number of these run concurrently against one
/// graph snapshot.
#[allow(clippy::too_many_arguments)]
fn find_candidates(
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
fn apply(
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
