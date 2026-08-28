//! Inverted-file search: scan a few clusters instead of the whole corpus.
//!
//! Exhaustive search costs `O(N)` per query, and for a large corpus that is the
//! whole cost. IVF partitions the rows once, then scans only the handful of
//! clusters nearest the query — trading a little recall for a large constant
//! factor.
//!
//! Rows are scored **exactly** within the lists that are scanned, so the only
//! error IVF makes is a true neighbour sitting in a list that was not probed.
//! That is what `nprobe` buys back, and why it is the dial a recall curve is
//! swept on. Compressing the rows themselves is product quantization, a
//! separate concern that composes on top of this one.

mod params;
mod pq;
mod pq_search;
mod quantizer;

pub use params::IvfParams;
pub use pq::IvfPqIndex;
pub use quantizer::Coarse;

use crate::domain::{Candidate, TopK};
use crate::ports::VectorIndex;
use std::time::Instant;
use telividb_core::{Ordinal, Result, VectorStore};
use telividb_distance::Scorer;
use telividb_telemetry::{Meter, fields, logger, metrics_names};

/// An IVF index with exact scoring inside each probed list.
pub struct IvfFlatIndex {
    coarse: Coarse,
    /// Row ids per cluster. The index *is* this partition.
    lists: Vec<Vec<u32>>,
    params: IvfParams,
    /// Where search measurements go.
    ///
    /// On the index rather than in the search signature because `search` takes
    /// `&self` and runs concurrently on a shared index. Disabled by default, so
    /// building one needs no pipeline and no runtime.
    meter: Meter,
}

impl IvfFlatIndex {
    /// Train centroids over `store` and assign every present row to a list.
    pub fn build(store: &dyn VectorStore, params: IvfParams) -> Result<Self> {
        let started = Instant::now();
        let coarse = Coarse::train(store, &params)?;

        let mut lists = vec![Vec::new(); coarse.len()];
        if !coarse.is_empty() {
            for row in 0..store.len() {
                // Absent is normal in a multimodal collection: a text-only
                // point has no image vector, and it belongs to no list.
                let Some(vector) = store.get(Ordinal::from_row(row as u32)) else {
                    continue;
                };
                lists[coarse.assign(vector)].push(row as u32);
            }
        }

        let elapsed = started.elapsed().as_secs_f64();
        let assigned: usize = lists.iter().map(Vec::len).sum();
        logger::info!("ivf built").with_data(&serde_json::json!({
            fields::INDEX_KIND: "ivf-flat",
            fields::ROWS: assigned,
            fields::LEVELS: coarse.len(),
            fields::DURATION_SECONDS: elapsed,
        }));

        Ok(Self {
            coarse,
            lists,
            params,
            meter: Meter::disabled(),
        })
    }

    /// Search `nprobe` lists instead, without rebuilding.
    ///
    /// `nprobe` is query-time — it changes nothing stored — so this is the axis
    /// a recall-versus-throughput curve sweeps.
    pub fn with_nprobe(mut self, nprobe: usize) -> Self {
        self.params = self.params.with_nprobe(nprobe);
        self
    }

    /// Record search measurements through `meter`.
    pub fn with_meter(mut self, meter: Meter) -> Self {
        self.meter = meter;
        self
    }

    /// The settings this index was built and searches with.
    pub fn params(&self) -> IvfParams {
        self.params
    }

    /// How many rows each list holds.
    ///
    /// Exposed because balance is what decides whether IVF helps: one list
    /// holding most of the corpus means a probe scans nearly everything, and
    /// that is invisible in a recall number alone.
    pub fn list_sizes(&self) -> Vec<usize> {
        self.lists.iter().map(Vec::len).collect()
    }
}

impl VectorIndex for IvfFlatIndex {
    fn kind(&self) -> &'static str {
        "ivf-flat"
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
            return Err(telividb_core::Error::DimMismatch {
                expected: dim,
                actual: query.len(),
            });
        }
        if k == 0 || self.coarse.is_empty() {
            return Ok(Vec::new());
        }

        let started = Instant::now();
        let metric = store.metric();
        let mut best = TopK::new(k, metric.higher_is_nearer());
        let mut visited = 0u64;

        for list in self.coarse.probe(query, metric, self.params.nprobe) {
            for &row in &self.lists[list] {
                let ordinal = Ordinal::from_row(row);

                // Consulted *during* the scan, never after. Filtering results
                // afterwards would leak how many rows were hidden and where
                // they ranked — invariant 15.
                if let Some(is_allowed) = allowed
                    && !is_allowed(ordinal)
                {
                    continue;
                }
                let Some(candidate) = store.get(ordinal) else {
                    continue;
                };

                best.offer(Candidate::new(ordinal, metric.score(query, candidate)));
                visited += 1;
            }
        }

        let elapsed = started.elapsed().as_secs_f64();
        self.meter
            .histogram(metrics_names::SEARCH_DURATION, elapsed);
        logger::debug!("ivf search").with_data(&serde_json::json!({
            fields::INDEX_KIND: "ivf-flat",
            fields::EF: self.params.nprobe,
            fields::CANDIDATES_VISITED: visited,
            fields::DURATION_SECONDS: elapsed,
        }));

        Ok(best.into_sorted())
    }
}

#[cfg(test)]
#[path = "ivf_test.rs"]
mod tests;
