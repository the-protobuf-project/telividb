//! Search algorithms, behind one port.
//!
//! Indexes never touch files — they see a [`VectorStore`], so storage layout
//! and search algorithm evolve independently. That separation is what "bring
//! your own search algorithm" actually means. See CLAUDE.md invariant 6.
#![forbid(unsafe_code)]

pub mod adapters;
pub mod domain;
pub mod ports;
pub mod recall;

pub use adapters::{FlatIndex, HnswIndex, HnswParams};
pub use domain::{
    Candidate, Hit, MergeStats, Merged, OverFetch, RerankStats, Source, TwoTierStats, merge_top_k,
    rerank, rerank_candidates, rerank_measured, two_tier_search,
};
pub use ports::{VectorIndex, VectorStore};
pub use recall::{RecallReport, recall_at_k};
