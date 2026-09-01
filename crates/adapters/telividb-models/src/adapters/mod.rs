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
mod store_receipt;
#[cfg(test)]
#[path = "store_receipt_test.rs"]
mod store_receipt_tests;

pub mod huggingface;

pub use gguf::GgufHeader;
#[cfg(feature = "network")]
pub use net::HttpFetcher;
pub use store::ModelStore;
