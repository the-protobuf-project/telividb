//! Batched parallel construction.
//!
//! Kept apart from the sequential insert path because it answers a different
//! question: not "how is a node linked" but "how is that work divided without
//! making the result depend on scheduling".
//!
//! Every node in a batch searches the same read-only snapshot concurrently;
//! their connections are then applied in row order on one thread. That ordering
//! is what makes a batched build reproducible.

mod helpers;

use super::graph::Graph;
use super::params::HnswParams;
use super::visited::VisitedSet;
use episteme_core::{Metric, Ordinal, VectorStore};
use helpers::{apply, find_candidates};

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

use super::scored::Scored;
