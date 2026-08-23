//! Answering a query against the graph.
//!
//! The [`VectorIndex`] implementation, kept apart from construction because
//! it answers a different question: not how the graph is built, but how a
//! descent through it is bounded, filtered and reported.

use super::scored::{Scored, from_distance};
use super::search::{distance_to, greedy_descend, search_layer};
use super::{HnswIndex, VectorIndex};
use crate::domain::Candidate;
use crate::ports::VectorStore;
use episteme_core::{Ordinal, Result};
use episteme_telemetry::{fields, logger, metrics_names, redact};
use std::time::Instant;

impl VectorIndex for HnswIndex {
    fn kind(&self) -> &'static str {
        "hnsw"
    }

    fn search(
        &self,
        store: &dyn VectorStore,
        query: &[f32],
        k: usize,
        allowed: Option<&dyn Fn(Ordinal) -> bool>,
    ) -> Result<Vec<Candidate>> {
        let dim = store.dim().get();
        if query.len() != dim {
            return Err(episteme_core::Error::DimMismatch {
                expected: dim,
                actual: query.len(),
            });
        }

        let ef = self.params.effective_ef(k);
        let started = Instant::now();

        let metric = store.metric();
        // An empty index legitimately has no entry point and no results.
        let Some(entry) = self.graph.entry() else {
            return Ok(Vec::new());
        };
        // An entry point that cannot be scored is not an empty collection — it
        // is a graph whose descent cannot start, so every row is unreachable.
        // Returning `Ok(vec![])` made that indistinguishable from "nothing
        // matched", which is the same failure invariant 27 rules out for a
        // locked vault: a caller must be able to tell "no results" from "no
        // results I could compute".
        let Some(entry_distance) = distance_to(store, metric, query, entry) else {
            return Err(episteme_core::Error::MalformedIndex {
                reason: "hnsw entry point has no vector for this field",
            });
        };

        let mut cursor = Scored::new(entry_distance, entry);
        for layer in (1..=self.graph.max_level()).rev() {
            cursor = greedy_descend(&self.graph, store, metric, query, cursor, layer);
        }

        // Borrowed from the pool rather than allocated here: a fresh set zeroes
        // four bytes per row on every query, which on a large field is more
        // work than the search it is bookkeeping for.
        let mut visited = self.scratch.take(store.len());
        let found = search_layer(
            &self.graph,
            store,
            metric,
            query,
            cursor,
            ef,
            0,
            allowed,
            &mut visited,
        );
        self.scratch.give_back(visited);
        let hits: Vec<Candidate> = found
            .into_iter()
            .take(k)
            .map(|s| Candidate::new(s.ordinal, from_distance(metric, s.distance)))
            .collect();

        let elapsed = started.elapsed().as_secs_f64();
        // The stack's metrics take no attributes, so the index kind travels on
        // the log record rather than as a metric dimension.
        self.meter
            .histogram(metrics_names::SEARCH_DURATION, elapsed);
        self.meter
            .histogram(metrics_names::SEARCH_RESULTS, hits.len() as f64);
        logger::debug!("search complete").with_data(&serde_json::json!({
            fields::INDEX_KIND: "hnsw",
            fields::K: k,
            fields::EF: ef,
            fields::FILTERED: allowed.is_some(),
            fields::QUERY: redact::vector_shape(query),
            fields::RESULTS_RETURNED: hits.len(),
            fields::DURATION_SECONDS: elapsed,
        }));
        Ok(hits)
    }
}
