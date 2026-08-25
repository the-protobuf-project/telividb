//! The search algorithm port.

use crate::domain::Candidate;
use crate::ports::VectorStore;
use telividb_core::Result;

/// A search algorithm over one named vector field.
///
/// Implementors are selected by configuration. Because this trait is the
/// extension point for custom algorithms and lives on the hot path, it is
/// compile-time only — a sandboxed plugin boundary crossed once per distance
/// computation would dominate every query. See ARCHITECTURE.md §8.1.
pub trait VectorIndex: Send + Sync {
    /// Human-readable algorithm name, as used in configuration.
    fn kind(&self) -> &'static str;

    /// Return the `k` nearest rows to `query`, best first.
    ///
    /// `allowed`, when present, is the visibility-and-filter bitmap resolved
    /// before the search: implementors must restrict traversal to those rows
    /// rather than filtering results afterwards. Post-filtering leaks the
    /// existence and ranking of rows the caller may not see — see
    /// ARCHITECTURE.md §6 and CLAUDE.md invariant 15.
    fn search(
        &self,
        store: &dyn VectorStore,
        query: &[f32],
        k: usize,
        allowed: Option<&dyn Fn(telividb_core::Ordinal) -> bool>,
    ) -> Result<Vec<Candidate>>;

    /// Answer several queries against the same corpus, in input order.
    ///
    /// **Why the port carries this at all.** One query against an exhaustive
    /// corpus is a matrix-vector product, which cannot fill a GPU — the device
    /// spends most of the call in dispatch overhead. Many queries at once is a
    /// matrix-matrix product, which can. Measured on a million 128-dimension
    /// rows, a batch of 32 costs 0.409 ms per query against 2.232 ms answered
    /// one at a time: 5.5&times; for the same work and the same results.
    ///
    /// A server answering concurrent requests already has the batch; without a
    /// method to hand it over, it has no way to say so.
    ///
    /// The default answers them one at a time, which is correct for every index
    /// and is what a graph-based one wants anyway — its traversal is sequential
    /// and shares nothing between queries. Only an index that can express the
    /// batch as a single larger operation should override it.
    fn search_batch(
        &self,
        store: &dyn VectorStore,
        queries: &[&[f32]],
        k: usize,
        allowed: Option<&dyn Fn(telividb_core::Ordinal) -> bool>,
    ) -> Result<Vec<Vec<Candidate>>> {
        queries
            .iter()
            .map(|query| self.search(store, query, k, allowed))
            .collect()
    }
}
