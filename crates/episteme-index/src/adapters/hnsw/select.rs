//! Choosing which neighbours to keep.

use super::scored::{Scored, to_distance};
use episteme_core::{Metric, VectorStore};

/// Select up to `m` neighbours from `candidates`, favouring diversity.
///
/// Keeping simply the `m` nearest is the obvious approach and it builds a badly
/// connected graph: in a cluster, every node's neighbours are other members of
/// the same cluster, so there is no edge out and search cannot cross between
/// regions.
///
/// The heuristic from the HNSW paper fixes this. A candidate is kept only if it
/// is closer to the query node than to any neighbour already selected —
/// so an edge is added only when it reaches somewhere the existing edges do not.
/// This is most of what separates a graph with 0.95 recall from one with 0.6.
///
/// `candidates` must be sorted nearest-first.
pub fn select_neighbours(
    store: &dyn VectorStore,
    metric: Metric,
    candidates: &[Scored],
    m: usize,
) -> Vec<u32> {
    let mut selected: Vec<Scored> = Vec::with_capacity(m);

    // The vectors of everything selected so far, held alongside.
    //
    // The dominance check below runs once per (candidate, selected) pair —
    // thousands of times per insert — and fetching a kept vector through the
    // store on every comparison put a bounds check and a slice construction
    // inside the innermost loop. Caching at most `m` vectors turns it into
    // arithmetic. Nothing about the result changes; only the cost does.
    let mut selected_vectors: Vec<&[f32]> = Vec::with_capacity(m);

    for &candidate in candidates {
        if selected.len() >= m {
            break;
        }

        let Some(vector) = store.get(candidate.ordinal) else {
            continue;
        };

        // Keep it only if it is nearer to the query than to anything already
        // chosen — otherwise an existing edge already covers that direction.
        let dominated = selected_vectors.iter().any(|kept| {
            to_distance(metric, episteme_distance::score(metric, vector, kept)) < candidate.distance
        });

        if !dominated {
            selected.push(candidate);
            selected_vectors.push(vector);
        }
    }

    // The heuristic can be stricter than the budget allows. Backfill with the
    // nearest rejected candidates rather than leaving a node under-connected,
    // which would cost more recall than the lost diversity gains.
    if selected.len() < m {
        for &candidate in candidates {
            if selected.len() >= m {
                break;
            }
            if !selected.iter().any(|s| s.ordinal == candidate.ordinal) {
                selected.push(candidate);
            }
        }
    }

    selected.into_iter().map(|s| s.ordinal.row()).collect()
}
