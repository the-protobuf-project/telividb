//! Query planning: composing retrieval out of the pieces beneath it.
//!
//! One operation today, and it is the one that decides what this database is.
//! A vector index answers *what is similar*; a graph answers *what is related*.
//! Retrieval-augmented generation over a knowledge graph needs the join of the
//! two — seed by similarity, expand along typed edges, then rank the union —
//! and neither half can produce it alone.
//!
//! # Why this is `domain/` and not an adapter
//!
//! It composes **results**, not indexes. The caller runs its own search and
//! hands the hits in, so nothing here names a vector index, a store or a
//! device. That is what makes the planner testable with no I/O at all, and it
//! is why this crate can sit inward of everything it orchestrates.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod domain;

pub use domain::{Expansion, Retrieved, Seed};
