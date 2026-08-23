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
    /// Nodes inserted per parallel batch. **Defaults to 128.**
    ///
    /// Within a batch, nodes search concurrently against one graph snapshot and
    /// therefore cannot link to each other. Larger batches parallelise better
    /// and diverge slightly from a sequential build.
    ///
    /// Deterministic at any value: batches are fixed by row order, so the
    /// result never depends on thread count or scheduling. The first batch is
    /// inserted sequentially, because a batch searching an empty graph finds
    /// nothing and links nothing.
    ///
    /// # What batching actually costs
    ///
    /// Measured on clustered vectors at 32 dimensions, `Metric::Cosine`,
    /// recall@10 over 40 queries, release build:
    ///
    /// | `batch_size` | 3k rows | 20k rows | 20k build |
    /// |---|---|---|---|
    /// | 1 | 1.0000 | 1.0000 | 2.80s |
    /// | 32 | 1.0000 | 1.0000 | 2.52s |
    /// | 64 | 0.9850 | 0.9975 | 2.30s |
    /// | 128 | 0.9850 | 0.9975 | 2.36s |
    /// | 256 | 0.9825 | 0.9975 | 2.22s |
    /// | 512 | 1.0000 | 1.0000 | 2.08s |
    /// | 1024 | 0.9975 | 1.0000 | 2.13s |
    ///
    /// Recall is flat within noise — the spread is a handful of judgements out
    /// of four hundred, and it does not increase with batch size. An earlier
    /// version of this table showed recall falling to 0.94 and attributed it to
    /// nodes within a batch being unable to link to each other. That was not
    /// the cause: every node in the *first* batch searched an empty graph, was
    /// pushed with no edges, and was never reachable again. The curve was one
    /// bug, measured. See `hnsw_parallel::no_batch_size_orphans_a_present_row`.
    ///
    /// # Why the default is not larger
    ///
    /// The speedup is real but modest — about 1.25x at 20k rows — because only
    /// the *search* half of an insert parallelises. Neighbour selection and
    /// pruning mutate the graph, so they stay sequential and cap the whole
    /// thing. That part of the original rationale was right: this is Amdahl,
    /// not a tuning problem, and **SIMD distance kernels are the lever that
    /// pays**, since they speed up both halves.
    ///
    /// 128 takes most of the available speedup while keeping each batch small
    /// enough that the intra-batch linking gap stays a rounding error. Raising
    /// it further buys little; `hnsw_parallel` holds every size to a 0.97 floor
    /// so a regression at any of them fails rather than passing on a relative
    /// comparison.
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
            batch_size: 128,
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
