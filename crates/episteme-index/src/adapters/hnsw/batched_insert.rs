//! The two phases of one node's insert, split so they can run in different
//! concurrency regimes.
//!
//! A sequential insert searches and links in one pass. A batched build cannot:
//! searching is read-only and wants every core, linking mutates shared state and
//! must happen in one fixed order. Separating the phases is what lets
//! [`super::batched`] schedule them differently while the result stays the same
//! as if each node had been inserted alone.
//!
//! Splitting also creates the hazard both functions are written against: the
//! candidates a node found describe a *snapshot*, while `apply` runs against a
//! graph that batch-mates have since mutated. Nothing derived from the snapshot
//! may be re-read from the live graph — hence the explicit layer tags below.

use super::build::prune;
use super::graph::Graph;
use super::params::HnswParams;
use super::scored::Scored;
use super::search::{distance_to, greedy_descend, search_layer};
use super::select::select_neighbours;
use super::visited::VisitedSet;
use episteme_core::{Metric, VectorStore};

/// Candidate neighbours for one node, tagged with the layer each list belongs to.
///
/// The layer is carried rather than re-derived: [`apply`] reads a graph that
/// earlier nodes in the same batch have already mutated, so anything inferred
/// from its current top layer disagrees with what the snapshot search actually
/// walked.
pub(super) type LayerCandidates = Vec<(usize, Vec<Scored>)>;

/// The read-only half of an insert: candidates for each layer this node joins.
///
/// Touches nothing mutable, so any number of these run concurrently against one
/// graph snapshot.
#[allow(clippy::too_many_arguments)]
pub(super) fn find_candidates(
    graph: &Graph,
    store: &dyn VectorStore,
    metric: Metric,
    params: &HnswParams,
    vector: &[f32],
    level: usize,
    visited: &mut VisitedSet,
) -> LayerCandidates {
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

    // Each list is tagged with the layer it was searched at. `apply` used to
    // re-derive that from the list's position and its own `graph.max_level()`,
    // which is read *after* earlier nodes in the same batch have been applied —
    // so a batch-mate that raised the top layer shifted every index, and a
    // node's layer-0 candidates were written to layer 1 instead. It was then
    // unreachable at the layer every search ends on.
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
        per_layer.push((layer, candidates));
    }
    per_layer
}

/// The mutating half of an insert: link the node using candidates already found.
pub(super) fn apply(
    graph: &mut Graph,
    store: &dyn VectorStore,
    metric: Metric,
    params: &HnswParams,
    level: usize,
    per_layer: LayerCandidates,
) {
    let node = graph.push_node(level);

    for (layer, candidates) in per_layer {
        if candidates.is_empty() {
            continue;
        }
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
