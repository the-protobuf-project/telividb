//! The manifest — the collection's single mutable pointer.

#[allow(clippy::module_inception)]
mod manifest;

pub use manifest::{MANIFEST_VERSION, Manifest};
