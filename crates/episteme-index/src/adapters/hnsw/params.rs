//! Tuning knobs, and what each one trades away.

/// HNSW construction and search parameters.
///
/// The three that matter, and the direction each pulls:
///
/// - `m` — neighbours per node. Larger means better recall and more memory,
///   since the graph is `m * 2 * 4` bytes per node at layer zero.
/// - `ef_construction` — candidate breadth while inserting. Larger means a
///   better graph and a slower build. Paid once.
/// - `ef_search` — candidate breadth while querying. Larger means better recall
///   and a slower query. Paid on every request, so this is the one to tune.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HnswParams {
    /// Maximum neighbours per node above layer zero.
    pub m: usize,
    /// Maximum neighbours at layer zero, conventionally `2 * m`.
    ///
    /// Layer zero holds every node and carries most of the search, so it is
    /// given more room than the sparse upper layers.
    pub m0: usize,
    /// Candidate breadth while inserting. Larger builds a better graph and
    /// costs build time, paid once.
    pub ef_construction: usize,
    /// Candidate breadth while querying. Larger improves recall and costs
    /// latency on every request — the lever to tune.
    pub ef_search: usize,
    /// Nodes inserted per parallel batch. **Defaults to one — no batching.**
    ///
    /// Within a batch, nodes search concurrently against one graph snapshot and
    /// therefore cannot link to each other. Larger batches parallelise better
    /// and diverge further from a sequential build.
    ///
    /// Deterministic at any value: batches are fixed by row order, so the result
    /// never depends on thread count or scheduling.
    ///
    /// # Why the default is one
    ///
    /// Measured on 20k clustered vectors at 128 dimensions:
    ///
    /// | `batch_size` | build | recall@10 |
    /// |---|---|---|
    /// | 1 | 7.98s | **1.0000** |
    /// | 32 | 7.56s | 0.9970 |
    /// | 128 | 6.85s | 0.9860 |
    /// | 512 | 6.32s | 0.9430 |
    ///
    /// A 1.26x speedup for 5.7 points of recall is a bad trade, and the shape of
    /// the curve says why: only the *search* half of an insert parallelises.
    /// Neighbour selection and pruning mutate the graph, so they stay
    /// sequential and cap the whole thing — Amdahl, not a tuning problem.
    ///
    /// **The lever that actually pays here is SIMD distance kernels**, which
    /// speed up both halves and cost no recall at all. Raise this only when
    /// build latency matters more than the last point of recall.
    pub batch_size: usize,

    /// Seed for level assignment.
    ///
    /// Fixed rather than drawn from the system, so a build is reproducible: a
    /// recall regression must be attributable to a code change, not to which
    /// levels the nodes happened to land on.
    pub seed: u64,
}

impl Default for HnswParams {
    fn default() -> Self {
        Self {
            m: 16,
            m0: 32,
            ef_construction: 200,
            ef_search: 64,
            batch_size: 1,
            seed: 0x5eed_1234_abcd_ef01,
        }
    }
}

impl HnswParams {
    /// Neighbour budget at `level`.
    pub fn max_neighbours(&self, level: usize) -> usize {
        if level == 0 { self.m0 } else { self.m }
    }

    /// Level-assignment normalisation factor, `1 / ln(m)`.
    ///
    /// Produces the exponential decay that makes each layer roughly `1/m` the
    /// size of the one below, which is what gives the descent its logarithmic
    /// behaviour.
    pub fn level_factor(&self) -> f64 {
        1.0 / (self.m as f64).ln()
    }

    /// `ef` used while querying, never below `k` — asking for more results than
    /// the candidate list can hold would silently cap the answer.
    pub fn effective_ef(&self, k: usize) -> usize {
        self.ef_search.max(k)
    }
}

#[cfg(test)]
#[path = "params_test.rs"]
mod tests;
