//! The coarse quantizer: which cluster a vector belongs to.
//!
//! Centroids are trained once per index and never mutate afterwards. That is
//! not incidental — CLAUDE.md forbids cross-segment state precisely here: a
//! centroid table shared or updated across segments breaks scatter-gather and
//! forecloses clustering later. Each index owns its own, and a query probes
//! every segment independently.

use super::params::IvfParams;
use telividb_core::{Metric, Ordinal, Result, VectorStore};
use telividb_distance::Scorer;
use telividb_distance::kmeans::KMeans;

/// Trained centroids, laid out contiguously as `nlist * dim` floats.
#[derive(Debug, Clone)]
pub struct Coarse {
    centroids: Vec<f32>,
    dim: usize,
}

impl Coarse {
    /// Train centroids over a deterministic sample of `store`.
    pub fn train(store: &dyn VectorStore, params: &IvfParams) -> Result<Self> {
        let dim = store.dim().get();
        let rows = store.len();

        // Sampled with a stride rather than at random: it is deterministic,
        // needs no shuffle buffer over a corpus that may not fit in memory, and
        // spreads the sample across the whole store rather than favouring a
        // prefix that may be ordered.
        let wanted = params.sample.min(rows).max(1);
        let stride = (rows / wanted).max(1);

        let mut owned: Vec<Vec<f32>> = Vec::new();
        let mut row = 0usize;
        while row < rows && owned.len() < wanted {
            if let Some(vector) = store.get(Ordinal::from_row(row as u32)) {
                owned.push(vector.to_vec());
            }
            row += stride;
        }

        // Every row absent is a legitimate state — a field no point populates —
        // and it produces an index that matches nothing rather than an error.
        if owned.is_empty() {
            return Ok(Self {
                centroids: Vec::new(),
                dim,
            });
        }

        // Never more clusters than training points: k-means would leave the
        // excess centroids on arbitrary reseeded points, and those lists would
        // then attract rows for no reason other than where a seed landed.
        let nlist = params.nlist.clamp(1, owned.len());
        let borrowed: Vec<&[f32]> = owned.iter().map(Vec::as_slice).collect();

        Ok(Self {
            centroids: KMeans::new(dim, nlist)
                .iterations(params.iterations)
                .seed(params.seed)
                .train(&borrowed),
            dim,
        })
    }

    /// How many clusters were trained.
    pub fn len(&self) -> usize {
        match self.dim {
            0 => 0,
            dim => self.centroids.len() / dim,
        }
    }

    /// Whether training produced nothing, which an empty store does.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The list a vector belongs to.
    ///
    /// By squared L2, matching how the centroids were trained. Assignment and
    /// training must agree: clustering under one measure and assigning under
    /// another puts rows in lists whose centroid does not describe them.
    pub fn assign(&self, vector: &[f32]) -> usize {
        KMeans::new(self.dim, self.len()).assign(vector, &self.centroids)
    }

    /// The `nprobe` lists nearest `query`, best first.
    ///
    /// Ranked by the collection's own metric rather than by L2. Probing is a
    /// *search* decision — it picks which lists could hold the true nearest
    /// neighbours — so it must use the measure the query is answered under, or
    /// it will confidently probe the wrong lists.
    pub fn probe(&self, query: &[f32], metric: Metric, nprobe: usize) -> Vec<usize> {
        let mut ranked: Vec<(usize, f32)> = self
            .centroids
            .chunks(self.dim.max(1))
            .enumerate()
            .map(|(list, centroid)| (list, metric.score(query, centroid)))
            .collect();

        let better = |a: &(usize, f32), b: &(usize, f32)| match metric.higher_is_nearer() {
            true => b.1.total_cmp(&a.1),
            false => a.1.total_cmp(&b.1),
        };

        let take = nprobe.min(ranked.len());
        if take < ranked.len() {
            ranked.select_nth_unstable_by(take, better);
            ranked.truncate(take);
        }
        ranked.sort_unstable_by(better);
        ranked.into_iter().map(|(list, _)| list).collect()
    }

    /// `vector - centroid[list]`, which is what PQ actually encodes.
    ///
    /// Residuals are small and centred on zero where absolute vectors are
    /// spread across the whole space, so the same code size carries far more
    /// information. Returns the vector unchanged when the list is out of
    /// range, which cannot happen for a list this quantizer assigned.
    pub fn residual(&self, vector: &[f32], list: usize) -> Vec<f32> {
        let start = list * self.dim;
        match self.centroids.get(start..start + self.dim) {
            Some(centroid) => vector.iter().zip(centroid).map(|(v, c)| v - c).collect(),
            None => vector.to_vec(),
        }
    }

    /// The trained centroids, for persistence.
    pub fn centroids(&self) -> &[f32] {
        &self.centroids
    }
}
