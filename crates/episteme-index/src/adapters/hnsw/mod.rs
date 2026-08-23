//! Hierarchical Navigable Small World graphs.
//!
//! The default index above trivial sizes. A multi-layer graph where upper
//! layers are sparse and act as an express lane: a query descends greedily
//! through them to land near the right region, then does a bounded best-first
//! search at layer zero where every node lives.
//!
//! Two implementation details carry most of the quality:
//!
//! - **The neighbour-selection heuristic** ([`select`]). Keeping the nearest
//!   `m` candidates builds a graph that cannot cross between clusters.
//! - **Filtering restricts what is kept, never what is traversed**
//!   ([`search`]). Refusing to walk through excluded nodes strands regions and
//!   silently costs recall.

mod build;
mod graph;
mod params;
mod rng;
mod scored;
mod search;
mod select;
mod serialize;
mod visited;

pub use graph::Graph;
pub use params::HnswParams;

use crate::domain::Candidate;
use crate::ports::{VectorIndex, VectorStore};
use episteme_core::{Ordinal, Result};
use episteme_telemetry::{fields, metrics_names, redact};
use scored::{Scored, from_distance};
use search::{distance_to, greedy_descend, search_layer};
use std::time::Instant;
use visited::VisitedSet;

/// An HNSW index over one named vector field.
#[derive(Debug)]
pub struct HnswIndex {
    graph: Graph,
    params: HnswParams,
}

impl HnswIndex {
    /// Build over every present row in `store`.
    pub fn build(store: &dyn VectorStore, params: HnswParams) -> Self {
        let span = tracing::info_span!(
            "episteme.index.build",
            { fields::INDEX_KIND } = "hnsw",
            { fields::ROWS } = store.len(),
        );
        let _guard = span.enter();
        let started = Instant::now();

        let graph = build::build(store, &params);

        metrics::histogram!(metrics_names::INDEX_BUILD_DURATION)
            .record(started.elapsed().as_secs_f64());
        tracing::info!(
            edges = graph.edge_count(),
            levels = graph.max_level() + 1,
            "hnsw built"
        );
        Self { graph, params }
    }

    /// The parameters this index was built and searches with.
    pub fn params(&self) -> HnswParams {
        self.params
    }

    /// The underlying proximity graph, for diagnostics and sizing.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Serialize the graph for `index.hnsw`.
    ///
    /// Only the graph is written. The vectors live in the segment's own files,
    /// so an index never duplicates them — and a rebuilt index over unchanged
    /// vectors is byte-identical, which makes the archive round-trip checkable.
    pub fn encode(&self) -> Vec<u8> {
        serialize::encode(&self.graph)
    }

    /// Reopen a graph written by [`HnswIndex::encode`].
    pub fn decode(bytes: &[u8], params: HnswParams) -> Result<Self> {
        Ok(Self {
            graph: serialize::decode(bytes)?,
            params,
        })
    }
}

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
        let span = tracing::debug_span!(
            "episteme.index.search",
            { fields::INDEX_KIND } = "hnsw",
            { fields::K } = k,
            ef,
            filtered = allowed.is_some(),
            query = %redact::vector_shape(query),
        );
        let _guard = span.enter();
        let started = Instant::now();

        let metric = store.metric();
        let Some(entry) = self.graph.entry() else {
            return Ok(Vec::new());
        };
        let Some(entry_distance) = distance_to(store, metric, query, entry) else {
            return Ok(Vec::new());
        };

        let mut cursor = Scored::new(entry_distance, entry);
        for layer in (1..=self.graph.max_level()).rev() {
            cursor = greedy_descend(&self.graph, store, metric, query, cursor, layer);
        }

        let mut visited = VisitedSet::with_capacity(store.len());
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
        let hits: Vec<Candidate> = found
            .into_iter()
            .take(k)
            .map(|s| Candidate::new(s.ordinal, from_distance(metric, s.distance)))
            .collect();

        let labels = [(fields::INDEX_KIND, "hnsw")];
        metrics::histogram!(metrics_names::SEARCH_DURATION, &labels)
            .record(started.elapsed().as_secs_f64());
        metrics::histogram!(metrics_names::SEARCH_RESULTS, &labels).record(hits.len() as f64);
        Ok(hits)
    }
}
