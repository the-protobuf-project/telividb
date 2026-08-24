//! A single indexed item.
//!
//! Carries a point's resource name, span, content reference, and its named
//! vector fields. **Not every point has every field** — a text-only point has
//! no image vector, and that absence is ordinary rather than exceptional
//! (invariant 17), which is why vectors are a map rather than a fixed slot.

use super::{ContentRef, ResourceName, Span};
use std::collections::BTreeMap;

/// One point, as it arrives and as it is stored.
///
/// Vectors are keyed by field name — `text_bge`, `image_clip` — because each
/// field has its own model, dimension and metric (ARCHITECTURE §4.1). A
/// `BTreeMap` rather than a `HashMap` so iteration order is stable, which
/// matters when the same point is written twice and the bytes should match.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// Resource name, e.g. `collections/media/points/doc-1`.
    pub name: ResourceName,
    /// Interval of the source media this point covers, if any.
    pub span: Option<Span>,
    /// Reference to the bytes this point was derived from, if any.
    pub content_ref: Option<ContentRef>,
    /// Named vector fields this point carries. Absent fields are simply not
    /// present, never zero-filled.
    pub vectors: BTreeMap<String, Vec<f32>>,
}

impl Point {
    /// A point with no span and no content reference.
    pub fn new(name: ResourceName) -> Self {
        Self {
            name,
            span: None,
            content_ref: None,
            vectors: BTreeMap::new(),
        }
    }

    /// Attach a vector for one named field.
    pub fn with_vector(mut self, field: impl Into<String>, vector: Vec<f32>) -> Self {
        self.vectors.insert(field.into(), vector);
        self
    }

    /// Attach a temporal span.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Attach a content reference.
    pub fn with_content_ref(mut self, content_ref: ContentRef) -> Self {
        self.content_ref = Some(content_ref);
        self
    }
}

#[cfg(test)]
#[path = "point_test.rs"]
mod tests;
