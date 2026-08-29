//! Implementations. One concern per file.

#[cfg(test)]
pub(crate) mod fake_fetcher;
#[cfg(feature = "network")]
mod net;
#[cfg(feature = "network")]
mod net_url;

mod gguf;
mod gguf_reader;
mod gguf_skip;
mod store;
mod store_install;

pub mod huggingface;

pub use gguf::GgufHeader;
#[cfg(feature = "network")]
pub use net::HttpFetcher;
pub use store::ModelStore;
