//! Search algorithms, behind one port.
//!
//! Indexes never touch files — they see a [`VectorStore`], so storage layout
//! and search algorithm evolve independently. That separation is what "bring
//! your own search algorithm" actually means. See CLAUDE.md invariant 6.
#![forbid(unsafe_code)]

pub mod adapters;
pub mod domain;
pub mod ports;

pub use adapters::FlatIndex;
pub use domain::{Candidate, Hit, MergeStats, Merged, Source, merge_top_k};
pub use ports::{VectorIndex, VectorStore};
