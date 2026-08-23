//! Descending the layers, and the bounded search at the bottom.

use super::graph::Graph;
use super::scored::{Farthest, Nearest, Scored, to_distance};
use super::visited::VisitedSet;
use episteme_core::{Metric, Ordinal, VectorStore};
use std::collections::BinaryHeap;

/// Distance from `query` to the vector at `ordinal`.
///
/// Returns `None` for a row with no value for this field — normal in a
/// multimodal collection, and it must not be scored as though it were zeros.
pub fn distance_to(
    store: &dyn VectorStore,
    metric: Metric,
    query: &[f32],
    ordinal: Ordinal,
) -> Option<f32> {
    let vector = store.get(ordinal)?;
    Some(to_distance(
        metric,
        episteme_distance::score(metric, query, vector),
    ))
}

/// Greedy descent through one upper layer.
///
/// Walks to a local minimum: repeatedly steps to the nearest neighbour that
/// improves on the current node, stopping when none does. Upper layers are
/// sparse, so this is cheap and only needs to land in the right region — the
/// bounded search at layer zero does the accurate work.
pub fn greedy_descend(
    graph: &Graph,
    store: &dyn VectorStore,
    metric: Metric,
    query: &[f32],
    entry: Scored,
    layer: usize,
) -> Scored {
    let mut current = entry;
    let mut improved = true;

    while improved {
        improved = false;
        for &neighbour in graph.neighbours(current.ordinal.row(), layer) {
            let candidate = Ordinal::from_row(neighbour);
            let Some(distance) = distance_to(store, metric, query, candidate) else {
                continue;
            };
            if distance < current.distance {
                current = Scored::new(distance, candidate);
                improved = true;
            }
        }
    }
    current
}

/// Bounded best-first search at one layer, returning up to `ef` candidates.
///
/// The two heaps do different jobs: `candidates` decides where to walk next,
/// `results` holds the best `ef` found so far. The search stops when the
/// nearest unexplored candidate is worse than the worst kept result — at that
/// point nothing reachable can improve the answer.
#[allow(clippy::too_many_arguments)]
pub fn search_layer(
    graph: &Graph,
    store: &dyn VectorStore,
    metric: Metric,
    query: &[f32],
    entry: Scored,
    ef: usize,
    layer: usize,
    allowed: Option<&dyn Fn(Ordinal) -> bool>,
    visited: &mut VisitedSet,
) -> Vec<Scored> {
    let mut candidates = BinaryHeap::new();
    let mut results: BinaryHeap<Farthest> = BinaryHeap::new();

    visited.clear();
    visited.visit(entry.ordinal.row() as usize);
    candidates.push(Nearest(entry));
    if is_visible(entry.ordinal, allowed) {
        results.push(Farthest(entry));
    }

    while let Some(Nearest(current)) = candidates.pop() {
        let worst = results.peek().map(|f| f.0.distance);
        if let Some(worst) = worst
            && current.distance > worst
            && results.len() >= ef
        {
            break;
        }

        for &neighbour in graph.neighbours(current.ordinal.row(), layer) {
            if !visited.visit(neighbour as usize) {
                continue;
            }

            let candidate = Ordinal::from_row(neighbour);
            let Some(distance) = distance_to(store, metric, query, candidate) else {
                continue;
            };
            let scored = Scored::new(distance, candidate);

            // Traversal is never restricted by the filter — only what is *kept*
            // is. Walking through excluded nodes is what keeps the graph
            // connected under a selective predicate; refusing to traverse them
            // strands whole regions and silently costs recall.
            candidates.push(Nearest(scored));

            if !is_visible(candidate, allowed) {
                continue;
            }
            if results.len() < ef {
                results.push(Farthest(scored));
            } else if let Some(worst) = results.peek()
                && distance < worst.0.distance
            {
                results.pop();
                results.push(Farthest(scored));
            }
        }
    }

    let mut out: Vec<Scored> = results.into_iter().map(|f| f.0).collect();
    out.sort_unstable();
    out
}

fn is_visible(ordinal: Ordinal, allowed: Option<&dyn Fn(Ordinal) -> bool>) -> bool {
    allowed.is_none_or(|f| f(ordinal))
}
