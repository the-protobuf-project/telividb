//! Choosing what to compact.
//!
//! Compaction is not free — it rewrites vectors and rebuilds indexes — so the
//! question is which segments repay that cost. Two conditions do:
//!
//! - **Tombstone ratio.** A segment whose rows are mostly deleted is being
//!   scanned for bytes that will be discarded. Rewriting it shrinks every
//!   subsequent query.
//! - **Small-segment count.** Many small segments mean many index descents per
//!   query, since each is searched independently. Merging trades one rewrite
//!   for a permanently cheaper fan-out.
//!
//! Deliberately *not* a background thread. Compaction is a function the host
//! calls, which keeps the core free of assumed threads — required if the
//! embedded path is ever to run somewhere without long-lived background compute
//! — and makes the whole thing testable without timing.

use crate::format::SegmentHeader;

/// When a segment is worth rewriting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompactionPolicy {
    /// Rewrite a segment once this fraction of its rows are tombstoned.
    pub tombstone_ratio: f64,
    /// Segments below this row count are candidates for merging.
    pub small_segment_rows: u64,
    /// Merge only once this many small segments have accumulated.
    pub min_merge_count: usize,
    /// Most segments one plan may name.
    ///
    /// Without a cap, every tombstone-heavy segment goes into a single plan —
    /// so a corpus with hundreds of them schedules one unbounded rewrite that
    /// reads the whole database, holds its output until the end, and cannot be
    /// checkpointed partway. Capping turns that into a sequence of runs, each
    /// of which publishes and can be interrupted.
    ///
    /// Segments are taken worst-first, so the rewrite that repays most happens
    /// first and the next run sees the rest.
    pub max_inputs: usize,
}

impl Default for CompactionPolicy {
    fn default() -> Self {
        Self {
            tombstone_ratio: 0.2,
            small_segment_rows: 10_000,
            min_merge_count: 4,
            max_inputs: 16,
        }
    }
}

/// What compaction should do, and why.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactionPlan {
    /// Segments to merge into one.
    pub inputs: Vec<u64>,
    /// Rows expected to survive, after dropping tombstones.
    pub surviving_rows: u64,
    /// Rows expected to be reclaimed.
    pub reclaimed_rows: u64,
    /// Human-readable justification, for logs and query explain.
    pub reason: &'static str,
}

impl CompactionPlan {
    /// Whether this plan would do any useful work.
    pub fn is_worthwhile(&self) -> bool {
        !self.inputs.is_empty()
    }
}

/// Decide what to compact, given each segment's id and header.
///
/// Returns `None` when nothing repays the rewrite. Tombstone pressure is
/// checked first: a segment mostly full of deleted rows costs every query,
/// where many small segments cost only fan-out.
pub fn plan(segments: &[(u64, SegmentHeader)], policy: CompactionPolicy) -> Option<CompactionPlan> {
    let mut heavy: Vec<&(u64, SegmentHeader)> = segments
        .iter()
        .filter(|(_, h)| h.rows > 0 && (h.deleted as f64 / h.rows as f64) >= policy.tombstone_ratio)
        .collect();

    if !heavy.is_empty() {
        // Worst first, then capped: one run should reclaim as much as it can
        // within a bounded rewrite, and leave the rest for the next.
        // `sort_by` with a total order on the ratio, and the segment id as the
        // tie-break so a plan is reproducible rather than dependent on the
        // sort's stability across inputs.
        heavy.sort_by(|(a_id, a), (b_id, b)| {
            let a_ratio = a.deleted as f64 / a.rows as f64;
            let b_ratio = b.deleted as f64 / b.rows as f64;
            b_ratio.total_cmp(&a_ratio).then(a_id.cmp(b_id))
        });
        heavy.truncate(policy.max_inputs.max(1));
        return Some(CompactionPlan {
            inputs: heavy.iter().map(|(id, _)| *id).collect(),
            surviving_rows: heavy.iter().map(|(_, h)| h.live_rows()).sum(),
            reclaimed_rows: heavy.iter().map(|(_, h)| h.deleted).sum(),
            reason: "tombstone ratio above threshold",
        });
    }

    let mut small: Vec<&(u64, SegmentHeader)> = segments
        .iter()
        .filter(|(_, h)| h.rows < policy.small_segment_rows)
        .collect();
    // Smallest first: merging the smallest segments removes the most fan-out
    // per byte rewritten. Capped for the same reason as the tombstone path.
    small.sort_by_key(|(id, h)| (h.rows, *id));
    small.truncate(policy.max_inputs.max(1));

    if small.len() >= policy.min_merge_count {
        return Some(CompactionPlan {
            inputs: small.iter().map(|(id, _)| *id).collect(),
            surviving_rows: small.iter().map(|(_, h)| h.live_rows()).sum(),
            reclaimed_rows: small.iter().map(|(_, h)| h.deleted).sum(),
            reason: "too many small segments",
        });
    }

    None
}

#[cfg(test)]
#[path = "plan_test.rs"]
mod tests;
