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

#[cfg(feature = "parallel")]
mod batched;
mod build;
mod cursor;
mod graph;
mod params;
mod query;
mod rng;
mod scored;
mod search;
mod select;
mod serialize;
mod visited;

pub use graph::Graph;
pub use params::HnswParams;

use crate::ports::{VectorIndex, VectorStore};
use episteme_core::Result;
use episteme_telemetry::{Meter, fields, logger, metrics_names};
use std::time::Instant;
use visited::ScratchPool;

/// An HNSW index over one named vector field.
#[derive(Debug)]
pub struct HnswIndex {
    graph: Graph,
    params: HnswParams,
    /// Where search measurements go.
    ///
    /// On the index rather than in the search signature because `search` takes
    /// `&self` and runs concurrently on a shared index — there is nowhere in
    /// that call to pass `&mut` state. Disabled by default, so building an
    /// index needs no pipeline and no runtime.
    meter: Meter,
    /// Reusable visited sets, one per concurrent search.
    scratch: ScratchPool,
}

impl HnswIndex {
    /// Build over every present row in `store`.
    pub fn build(store: &dyn VectorStore, params: HnswParams) -> Self {
        Self::build_with_meter(store, params, Meter::disabled())
    }

    /// Build over every present row in `store`, reporting through `meter`.
    pub fn build_with_meter(store: &dyn VectorStore, params: HnswParams, meter: Meter) -> Self {
        let started = Instant::now();

        let graph = build::build(store, &params);

        let elapsed = started.elapsed().as_secs_f64();
        meter.histogram(metrics_names::INDEX_BUILD_DURATION, elapsed);
        logger::info!("hnsw built").with_data(&serde_json::json!({
            fields::INDEX_KIND: "hnsw",
            fields::ROWS: store.len(),
            fields::EDGES: graph.edge_count(),
            fields::LEVELS: graph.max_level() + 1,
            fields::DURATION_SECONDS: elapsed,
        }));
        Self {
            graph,
            params,
            meter,
            scratch: ScratchPool::default(),
        }
    }

    /// Record search measurements through `meter`.
    pub fn with_meter(mut self, meter: Meter) -> Self {
        self.meter = meter;
        self
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
            meter: Meter::disabled(),
            scratch: ScratchPool::default(),
        })
    }
}
