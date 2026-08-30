//! Implementations. One concern per file.

#[cfg(feature = "keychain")]
mod keychain_store;
mod memory_store;

#[cfg(feature = "keychain")]
pub use keychain_store::KeychainStore;
pub use memory_store::MemoryStore;
