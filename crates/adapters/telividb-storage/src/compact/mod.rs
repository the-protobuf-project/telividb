//! Reclaiming space from tombstoned rows.

#[allow(clippy::module_inception)]
mod compact;
mod plan;

pub use compact::{CompactionResult, compact_field};
pub use plan::{CompactionPlan, CompactionPolicy, plan};
