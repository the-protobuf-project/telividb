//! A scored search result.

use episteme_core::Ordinal;

/// One hit, still identified by its segment-local [`Ordinal`].
///
/// Translation to an `ExternalId` happens above the index, on the way out — an
/// ordinal must never reach a caller. See CLAUDE.md invariant 9.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    /// Segment-local row this hit refers to.
    pub ordinal: Ordinal,
    /// Score on the field's own metric scale. Direction depends on the metric.
    pub score: f32,
}

impl Candidate {
    /// Pair a row with the score it earned.
    pub fn new(ordinal: Ordinal, score: f32) -> Self {
        Self { ordinal, score }
    }
}
