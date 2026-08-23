//! Inserting nodes into the graph.

use super::graph::Graph;
use super::params::HnswParams;
use super::rng::SplitMix64;
use super::scored::Scored;
use super::search::{distance_to, greedy_descend, search_layer};
use super::select::select_neighbours;
use super::visited::VisitedSet;
use episteme_core::{Metric, Ordinal, VectorStore};

/// Build a graph over every present row in `store`.
///
/// Insertion order is the store's row order, and level assignment is seeded, so
/// the same input always yields the same graph. That reproducibility is what
/// makes a recall regression attributable to a code change rather than to luck.
pub fn build(store: &dyn VectorStore, params: &HnswParams) -> Graph {
    let mut graph = Graph::new();
    let mut rng = SplitMix64::new(params.seed);
    let metric = store.metric();
    let factor = params.level_factor();
    // One allocation for the entire build rather than one per layer visit.
    let mut visited = VisitedSet::with_capacity(store.len());

    for row in 0..store.len() {
        let ordinal = Ordinal::from_row(row as u32);
        let Some(vector) = store.get(ordinal) else {
            // Absent for this field. It still occupies an ordinal so stride
            // holds, but it is not a node in the graph.
            graph.push_node(0);
            continue;
        };
        let level = rng.level(factor);
        insert(
            &mut graph,
            store,
            metric,
            params,
            vector,
            level,
            &mut visited,
        );
    }
    graph
}

#[allow(clippy::too_many_arguments)]
fn insert(
    graph: &mut Graph,
    store: &dyn VectorStore,
    metric: Metric,
    params: &HnswParams,
    vector: &[f32],
    level: usize,
    visited: &mut VisitedSet,
) {
    let previous_entry = graph.entry();
    let previous_max = graph.max_level();
    let node = graph.push_node(level);

    let Some(entry) = previous_entry else {
        return; // First node: it is the entry point and has nothing to link to.
    };
    let Some(entry_distance) = distance_to(store, metric, vector, entry) else {
        return;
    };

    // Descend the layers above this node's level, narrowing to a good region.
    let mut cursor = Scored::new(entry_distance, entry);
    for layer in ((level + 1)..=previous_max).rev() {
        cursor = greedy_descend(graph, store, metric, vector, cursor, layer);
    }

    // Connect at every layer this node occupies.
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
        if candidates.is_empty() {
            continue;
        }
        cursor = candidates[0];

        let budget = params.max_neighbours(layer);
        let chosen = select_neighbours(store, metric, &candidates, budget);
        graph.set_neighbours(node, layer, chosen.clone());

        // Edges are undirected, so link back — and prune the neighbour if that
        // pushes it over budget. Dropping the new edge instead would make
        // connectivity depend on insertion order.
        for neighbour in chosen {
            if graph.try_connect(neighbour, layer, node, budget) {
                prune(graph, store, metric, neighbour, layer, budget);
            }
        }
    }
}

/// Re-run the selection heuristic over a node's neighbours and keep the best.
fn prune(
    graph: &mut Graph,
    store: &dyn VectorStore,
    metric: Metric,
    node: u32,
    layer: usize,
    budget: usize,
) {
    let ordinal = Ordinal::from_row(node);
    let Some(vector) = store.get(ordinal) else {
        return;
    };

    let mut candidates: Vec<Scored> = graph
        .neighbours(node, layer)
        .iter()
        .filter_map(|&n| {
            let other = Ordinal::from_row(n);
            distance_to(store, metric, vector, other).map(|d| Scored::new(d, other))
        })
        .collect();
    candidates.sort_unstable();

    let kept = select_neighbours(store, metric, &candidates, budget);
    graph.set_neighbours(node, layer, kept);
}
