//! Pure types for the index layer.

mod candidate;
mod merge;

pub use candidate::Candidate;
pub use merge::{Hit, MergeStats, Merged, Source, merge_top_k};
