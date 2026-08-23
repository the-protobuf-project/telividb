//! Temporal spans for time-indexed media.
//!
//! A 90-minute recording is not one embedding. Retrieval returns *a moment*,
//! so media points carry a span. See ARCHITECTURE.md §4.2.

/// A half-open interval `[start_ms, end_ms)` in milliseconds from the start of
/// the source content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Inclusive start, in milliseconds from the beginning of the source.
    start_ms: u64,
    /// Exclusive end. Equal to `start_ms` for a zero-length span.
    end_ms: u64,
}

impl Span {
    /// Build a span, rejecting one whose end precedes its start.
    pub fn new(start_ms: u64, end_ms: u64) -> crate::Result<Self> {
        if end_ms < start_ms {
            return Err(crate::Error::InvalidSpan { start_ms, end_ms });
        }
        Ok(Self { start_ms, end_ms })
    }

    /// Inclusive start offset.
    pub fn start_ms(self) -> u64 {
        self.start_ms
    }

    /// Exclusive end offset.
    pub fn end_ms(self) -> u64 {
        self.end_ms
    }

    /// Length in milliseconds; zero for an empty span.
    pub fn duration_ms(self) -> u64 {
        self.end_ms - self.start_ms
    }

    /// Whether two spans share any instant. Half-open, so touching spans do not
    /// overlap — adjacent transcript segments are not "simultaneous".
    pub fn overlaps(self, other: Span) -> bool {
        self.start_ms < other.end_ms && other.start_ms < self.end_ms
    }

    /// Whether `other` lies entirely within `self`.
    pub fn contains(self, other: Span) -> bool {
        self.start_ms <= other.start_ms && other.end_ms <= self.end_ms
    }
}

#[cfg(test)]
#[path = "span_test.rs"]
mod tests;
