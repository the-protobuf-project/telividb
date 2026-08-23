//! The search algorithm port.

use crate::domain::Candidate;
use crate::ports::VectorStore;
use episteme_core::Result;

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
        allowed: Option<&dyn Fn(episteme_core::Ordinal) -> bool>,
    ) -> Result<Vec<Candidate>>;
}
