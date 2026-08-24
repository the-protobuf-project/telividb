//! A single indexed item, minus the vectors.
//!
//! Deliberately narrower than the `Point` message in `point.proto`: this
//! carries a point's resource name, span and content reference, but no named
//! vectors. Wiring vectors means engaging the segment/buffer machinery and
//! real schema resolution, neither of which exists yet — that lands with the
//! vector service, not here.

use super::{ContentRef, ResourceName, Span};

/// One point's non-vector fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// Resource name, e.g. `collections/media/points/doc-1`.
    pub name: ResourceName,
    /// Interval of the source media this point covers, if any.
    pub span: Option<Span>,
    /// Reference to the bytes this point was derived from, if any.
    pub content_ref: Option<ContentRef>,
}

impl Point {
    /// A point with no span and no content reference.
    pub fn new(name: ResourceName) -> Self {
        Self {
            name,
            span: None,
            content_ref: None,
        }
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
