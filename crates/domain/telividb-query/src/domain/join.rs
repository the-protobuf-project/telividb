//! Seeding by similarity, expanding along edges, and ranking the union.
//!
//! **This is the operation that makes the system a graph-RAG store** rather
//! than a vector index with a graph beside it. Neither half produces it: a
//! vector search finds what is *similar* to the query, a traversal finds what
//! is *related* to a thing, and the answer a retrieval-augmented prompt needs
//! is the union — ranked so that a strong direct match still outranks a weak
//! neighbour of a strong match.
//!
//! # Three decisions worth naming
//!
//! **A node reached twice keeps its best score, never the sum.** Summing
//! rewards nodes with many inbound edges — hubs — which is popularity, not
//! relevance, and it would put the most-connected entity at the top of every
//! result regardless of the query.
//!
//! **Seeds are never displaced by their own expansion.** A seed's score is
//! taken as searched; if the graph also reaches it, the direct hit wins.
//! Otherwise a document could be demoted for the crime of being well connected.
//!
//! **One visibility predicate covers both halves.** It is applied to the seeds
//! *and* threaded into the traversal, because a graph edge is a second path to
//! a row and invariant 34 requires both to be checked the same way. The
//! parameter exists now, before there is a policy engine to supply one, so that
//! wiring it in later is not a signature break.

use crate::domain::{Expansion, Retrieved, Seed};
use std::collections::HashMap;
use std::time::Instant;
use telividb_core::ResourceName;
use telividb_graph::Graph;
use telividb_telemetry::{fields, logger};

impl Expansion {
    /// Expand `seeds` through `graph` and rank the union, best first.
    ///
    /// `allowed`, when given, is checked against every resource before it is
    /// returned *or* expanded from: a node it rejects is treated as absent, so
    /// nothing beyond it is reachable either. That is what makes the graph path
    /// enforce the same visibility the search path does, rather than leaking a
    /// hidden row's existence through its neighbours.
    pub fn join(
        &self,
        seeds: &[Seed],
        graph: &Graph,
        allowed: Option<&dyn Fn(&ResourceName) -> bool>,
    ) -> Vec<Retrieved> {
        let started = Instant::now();

        // Seeds first, so a direct hit always wins a tie against the graph
        // reaching the same node.
        let mut best: HashMap<ResourceName, Retrieved> = HashMap::new();
        for seed in seeds {
            if let Some(is_allowed) = allowed
                && !is_allowed(&seed.name)
            {
                continue;
            }
            let entry = Retrieved {
                name: seed.name.clone(),
                score: seed.score,
                hops: 0,
                via: None,
            };
            best.entry(seed.name.clone())
                .and_modify(|held| {
                    if self.rank(entry.score, held.score) == std::cmp::Ordering::Less {
                        *held = entry.clone();
                    }
                })
                .or_insert(entry);
        }

        let expanded = self.expand(seeds, graph, allowed, &mut best);

        let mut out: Vec<Retrieved> = best.into_values().collect();
        // Ties break on the name so the order is reproducible across runs —
        // a `HashMap`'s iteration order is not, and a result set that reshuffles
        // between identical queries is indistinguishable from a ranking bug.
        out.sort_by(|a, b| {
            self.rank(a.score, b.score)
                .then_with(|| a.name.cmp(&b.name))
        });

        logger::debug!("graph join complete").with_data(&serde_json::json!({
            fields::RESULTS_RETURNED: out.len(),
            // Reported apart from the total: a join whose results are almost
            // all expansion is one where the seeding failed and the graph is
            // carrying the answer, which reads as "it worked" from the total
            // alone.
            fields::EXPANDED: expanded,
            fields::FILTERED: allowed.is_some(),
            fields::DURATION_SECONDS: started.elapsed().as_secs_f64(),
        }));
        out
    }

    /// Walk out from every seed, recording the best path to each node reached.
    ///
    /// Returns how many distinct nodes the traversal contributed, which is what
    /// the budget is spent on.
    fn expand(
        &self,
        seeds: &[Seed],
        graph: &Graph,
        allowed: Option<&dyn Fn(&ResourceName) -> bool>,
        best: &mut HashMap<ResourceName, Retrieved>,
    ) -> usize {
        if self.hops == 0 || self.max_expanded == 0 {
            return 0;
        }

        let mut contributed = 0usize;
        for seed in seeds {
            // Hop by hop rather than all at once, because the distance is what
            // sets the decay — a node found at two hops must not be scored as
            // though it were adjacent.
            for hop in 1..=self.hops {
                if contributed >= self.max_expanded {
                    return contributed;
                }
                let reached = graph.k_hop(&seed.name, hop, self.edge_type.as_deref(), allowed);
                for name in reached {
                    // Every node is offered and the comparison below decides.
                    //
                    // An earlier revision skipped anything already held at this
                    // distance or nearer, to avoid re-walking what a previous
                    // hop had seen — `k_hop(n)` returns everything *within* n
                    // edges, so each round re-reports the last one's nodes. But
                    // that guard also blocked a *different, better* seed from
                    // improving a node it had already reached, which is exactly
                    // the case this join exists to get right. The redundant
                    // work is bounded by the budget; the wrong score was not.
                    let candidate = Retrieved {
                        score: self.decayed(seed.score, hop),
                        name: name.clone(),
                        hops: hop,
                        via: Some(seed.name.clone()),
                    };

                    match best.get_mut(&name) {
                        // A seed is never displaced by the graph reaching it.
                        Some(held) if held.hops == 0 => continue,
                        Some(held) => {
                            if self.rank(candidate.score, held.score) == std::cmp::Ordering::Less {
                                *held = candidate;
                            }
                        }
                        None => {
                            if contributed >= self.max_expanded {
                                return contributed;
                            }
                            best.insert(name, candidate);
                            contributed += 1;
                        }
                    }
                }
            }
        }
        contributed
    }
}

#[cfg(test)]
#[path = "join_test.rs"]
mod tests;
