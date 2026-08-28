//! How an inverted-file index is shaped and searched.

/// Build- and query-time settings for [`super::IvfFlatIndex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IvfParams {
    /// How many clusters the corpus is partitioned into.
    ///
    /// The classic starting point is roughly `sqrt(rows)`: it balances the cost
    /// of scanning the centroids against the cost of scanning a list, since a
    /// query pays both. Too few and each list is nearly the whole corpus; too
    /// many and finding the right lists costs as much as scanning them.
    pub nlist: usize,

    /// How many of the nearest lists a query actually scans.
    ///
    /// The accuracy dial, and **query-time** — it changes no stored state, so a
    /// recall-versus-throughput curve is swept on it without rebuilding, exactly
    /// as `ef_search` is for HNSW.
    ///
    /// One is fast and misses whatever sits near a cluster boundary, which is
    /// most of the error IVF makes. Raising it recovers those neighbours and
    /// costs proportionally more scanning.
    pub nprobe: usize,

    /// Vectors sampled to train the centroids.
    ///
    /// Training on the whole corpus is wasteful — the centroids converge long
    /// before the last million points are consulted — so a sample stands in.
    /// Too small a sample gives centroids that describe the sample rather than
    /// the corpus, which shows up as badly unbalanced lists.
    pub sample: usize,

    /// Lloyd iterations during training.
    pub iterations: usize,

    /// Seed for sampling and k-means, so a build reproduces exactly.
    ///
    /// Reproducibility is not a nicety here: the list a row lands in is part of
    /// the index, so a nondeterministic build produces two indexes that answer
    /// the same query differently.
    pub seed: u64,
}

impl Default for IvfParams {
    /// Settings that behave reasonably before anything has been tuned.
    fn default() -> Self {
        Self {
            nlist: 256,
            nprobe: 8,
            sample: 65_536,
            // Few, because k-means converges quickly at this scale and each
            // extra pass costs a full sweep of the sample.
            iterations: 12,
            seed: 0x5EED_1F5F,
        }
    }
}

impl IvfParams {
    /// Settings scaled to a corpus of `rows`.
    ///
    /// `nlist` near `sqrt(rows)`, clamped so a tiny corpus does not end up with
    /// more clusters than vectors — which would leave most lists empty and make
    /// `nprobe` meaningless.
    pub fn for_rows(rows: usize) -> Self {
        let nlist = (rows as f64).sqrt().round().max(1.0) as usize;
        let nlist = nlist.clamp(1, rows.max(1));
        Self {
            nlist,
            nprobe: (nlist / 16).max(1),
            ..Self::default()
        }
    }

    /// The same settings, searching `nprobe` lists.
    pub fn with_nprobe(mut self, nprobe: usize) -> Self {
        self.nprobe = nprobe.max(1);
        self
    }
}
