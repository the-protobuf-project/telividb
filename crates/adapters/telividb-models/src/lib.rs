//! The curated model catalog, and acquiring the files it names.
//!
//! A person should never have to find a GGUF, judge whether it loads, and put
//! it somewhere the engine looks. They pick from a list, or paste a repository,
//! and the rest is this crate's problem.
//!
//! Three things it guarantees, in the order they matter:
//!
//! 1. **Nothing unloadable is offered or fetched.** The gate is
//!    [`Architecture`](telividb_core::Architecture), shared with the encoder,
//!    and it is checked before bytes move rather than after.
//! 2. **What arrives is what was curated.** Every download is verified against
//!    a SHA-256 recorded at curation time; a mismatch is refused, never loaded.
//! 3. **It works offline.** The catalog is compiled in, so the list is
//!    available with no network and cannot change under a running install.
//!
//! The network itself is a port ([`Fetcher`]), so everything above is testable
//! without one — and the HTTP adapter stays optional, which keeps this crate's
//! dependency surface small enough to publish on its own (rule 51).
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod adapters;
mod domain;
mod error;
mod ports;

pub use adapters::{GgufHeader, ModelStore, huggingface};
pub use domain::{Catalog, CatalogEntry};
pub use error::{Error, Result};
pub use ports::Fetcher;
