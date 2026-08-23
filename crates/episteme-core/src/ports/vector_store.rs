//! What an index is allowed to see of storage.
//!
//! This lives in `core` rather than in either neighbour because both implement
//! or consume it: storage's unsealed buffer *is* a vector store, and so is a
//! sealed segment, and the index must not know which one it was handed.

use crate::{Dim, Metric, Ordinal};

/// Read access to one named vector field within one segment.
///
/// Deliberately narrow. An index gets vectors and counts — never paths, never
/// file handles, never knowledge of quantization or mmap. Widening this trait
/// is how storage and search become coupled.
pub trait VectorStore {
    /// Width of every vector in this field.
    fn dim(&self) -> Dim;

    /// How similarity is measured in this field.
    fn metric(&self) -> Metric;

    /// Number of rows, including any that are tombstoned or absent.
    fn len(&self) -> usize;

    /// Whether this store holds no rows.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The vector at `ordinal`, or `None` if this row has no value for this
    /// field — which is normal in a multimodal collection, where a text-only
    /// point has no image vector. See ARCHITECTURE.md §4.1.
    fn get(&self, ordinal: Ordinal) -> Option<&[f32]>;
}
