//! Batched parallel construction.
//!
//! Kept apart from the sequential insert path because it answers a different
//! question: not "how is a node linked" but "how is that work divided without
//! making the result depend on scheduling".
//!
//! Every node in a batch searches the same read-only snapshot concurrently;
//! their connections are then applied in row order on one thread. That ordering
//! is what makes a batched build reproducible. The two halves themselves live in
//! [`super::batched_insert`] — this file owns only the division of work.

use super::batched_insert::{LayerCandidates, apply, find_candidates};
use super::build::insert_range;
use super::graph::Graph;
use super::params::HnswParams;
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
    let seeded = seed_entry(&mut graph, store, params, levels, metric, &mut visited);

    for start in (seeded..rows).step_by(params.batch_size) {
        let end = (start + params.batch_size).min(rows);

        // `map_init` builds the visited set once per worker thread rather than
        // once per node — the same allocation trap that made the sequential
        // build quadratic before.
        let found: Vec<Option<LayerCandidates>> = (start..end)
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
                    apply(&mut graph, store, metric, params, levels[row], per_layer);
                }
                // Absent for this field: it still occupies an ordinal so stride
                // holds, but it is not a node in the graph — and must never
                // become the entry point.
                _ => {
                    graph.push_absent();
                }
            }
        }
    }
    graph
}

/// Insert sequentially until the graph has an entry point; returns the first row
/// left for the parallel loop.
///
/// The first batch cannot be parallel, because there is nothing yet for a
/// concurrent search to find. Every node in batch zero would otherwise see an
/// empty snapshot, return no candidates, and be pushed with no edges — and
/// nothing links them afterwards, because nothing can reach them. That orphaned
/// exactly `batch_size - 1` nodes and cost recall in proportion: 0.57 at
/// batch_size 512 against 1.00 sequential.
///
/// Continues past absent rows until a valid entry is established or all rows are
/// consumed. An absent prefix longer than `batch_size` used to stop after the
/// first `batch_size` rows, leaving the graph with no entry point even when
/// present rows followed.
fn seed_entry(
    graph: &mut Graph,
    store: &dyn VectorStore,
    params: &HnswParams,
    levels: &[usize],
    metric: Metric,
    visited: &mut VisitedSet,
) -> usize {
    let rows = store.len();
    let mut seeded = 0;
    while seeded < rows && graph.entry().is_none() {
        let end = (seeded + params.batch_size).min(rows);
        insert_range(graph, store, params, metric, levels, seeded..end, visited);
        seeded = end;
    }
    seeded
}
