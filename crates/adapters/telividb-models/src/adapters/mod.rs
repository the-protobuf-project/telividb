//! Implementations. One concern per file.

#[cfg(test)]
pub(crate) mod fake_fetcher;
mod gguf;
mod gguf_reader;
mod store;

pub mod huggingface;

pub use gguf::GgufHeader;
pub use store::ModelStore;
