//! Which providers exist, and the keys that reach them.
//!
//! Answering itself is not here and is not in Rust. The provider SDKs worth
//! using are published by the model vendors as TypeScript, and the app already
//! runs TypeScript in its window — so the call is made there, directly, and this
//! crate holds the two things the window must not.
//!
//! 1. **The key store.** [`SecretStore`] writes to the OS keychain and has no
//!    listing method, so a caller can ask whether a provider is configured but
//!    never read back what was stored.
//! 2. **The provider table.** [`PROVIDERS`] is the one place a provider's name,
//!    display name and [`Locality`] are written down, so "is this local?" has a
//!    single answer rather than one per caller.
//!
//! [`may_answer`] encodes the rule that protected content is answered on this
//! machine or not at all. **Nothing calls it yet** — see its own documentation
//! for why that matters and what has to land before the rule is real.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod adapters;
mod domain;
mod error;
mod ports;

pub use domain::{Locality, PROVIDERS, Provider, may_answer, provider};
pub use error::{Error, Result};
pub use ports::SecretStore;

#[cfg(feature = "keychain")]
pub use adapters::KeychainStore;
pub use adapters::MemoryStore;
