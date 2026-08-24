//! Bounded breadth-first expansion from a seed resource.

use crate::Graph;
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, VecDeque};
use std::time::Instant;
use telividb_core::ResourceName;
use telividb_telemetry::{fields, logger, redact};

impl Graph {
    /// Every resource reachable from `seed` within `hops` edges.
    ///
    /// `edge_type`, when given, restricts which edges are followed —
    /// `Some("HAS_SHOT")` walks only that relationship, `None` walks all of
    /// them. `allowed`, when given, is checked before a node is added to the
    /// result *or* expanded further: a node this predicate rejects is treated
    /// as if it were not in the graph at all, so nothing beyond it is
    /// reachable either. Nothing calls this with `Some(..)` yet — there is no
    /// policy engine to supply one — but the parameter exists now so that
    /// wiring one in later is not a signature break. See CLAUDE.md rule 34
    /// and `telividb_index::VectorIndex::search`'s identical `allowed` shape.
    ///
    /// The seed itself is never included in the result, only what is reached
    /// from it. An unknown seed reaches nothing.
    pub fn k_hop(
        &self,
        seed: &ResourceName,
        hops: usize,
        edge_type: Option<&str>,
        allowed: Option<&dyn Fn(&ResourceName) -> bool>,
    ) -> Vec<ResourceName> {
        let started = Instant::now();
        let Some(&start) = self.index.get(seed) else {
            // Distinct from "reached nothing": an unknown seed usually means a
            // name was mistyped or the graph was never populated, and the two
            // look identical in an empty result.
            logger::debug!("traversal seed is not in the graph").with_data(&serde_json::json!({
                fields::RESOURCE: redact::resource_token(seed.as_str()),
            }));
            return Vec::new();
        };

        let mut visited = HashSet::new();
        visited.insert(start);
        let mut frontier = VecDeque::from([(start, 0usize)]);
        let mut reached = Vec::new();

        while let Some((node, depth)) = frontier.pop_front() {
            if depth >= hops {
                continue;
            }
            for edge_ref in self.inner.edges(node) {
                if edge_type.is_some_and(|wanted| edge_ref.weight().edge_type != wanted) {
                    continue;
                }
                let target = edge_ref.target();
                if visited.contains(&target) {
                    continue;
                }
                let name = &self.inner[target];
                if allowed.is_some_and(|is_allowed| !is_allowed(name)) {
                    continue;
                }
                visited.insert(target);
                reached.push(name.clone());
                frontier.push_back((target, depth + 1));
            }
        }

        logger::debug!("traversal complete").with_data(&serde_json::json!({
            fields::RESOURCE: redact::resource_token(seed.as_str()),
            fields::HOPS: hops,
            fields::EDGE_TYPE: edge_type.unwrap_or("*"),
            fields::FILTERED: allowed.is_some(),
            fields::NODES: reached.len(),
            fields::DURATION_SECONDS: started.elapsed().as_secs_f64(),
        }));
        reached
    }
}

#[cfg(test)]
#[path = "traversal_test.rs"]
mod tests;
