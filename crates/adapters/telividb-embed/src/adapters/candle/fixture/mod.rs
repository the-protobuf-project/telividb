//! A tiny, real GGUF encoder, built in memory for tests.
//!
//! **Why a real file rather than a mock.** Almost every way this crate can be
//! wrong is a disagreement with the GGUF format or with candle's loader:
//! a tensor named slightly differently, a dimension written in the other
//! order, a metadata key at the wrong integer width. A mocked loader agrees
//! with whatever the code already does and so tests none of that.
//!
//! The model is deliberately absurd — two layers, 8 hidden, 4 tokens — so the
//! whole suite runs in milliseconds while exercising the identical code path a
//! real 137M-parameter encoder takes.

mod build;
mod tensors;

pub use build::{TinyModel, write_tiny_gguf};
