//! Shared scaffolding for the examples.
//!
//! Only what more than one example needs. An example is meant to be read
//! top-to-bottom, so anything used once stays inline in the binary that uses
//! it rather than being hidden behind a helper here.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod corpus;
pub mod model;
pub mod report;
