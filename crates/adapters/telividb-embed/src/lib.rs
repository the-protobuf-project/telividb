//! The inference server: one place every embedding is computed.
//!
//! Not "the embedding crate" — the single compute boundary for the whole
//! system (rules 42–45). Ingest embedding, query-time encoding and, later,
//! every plugin's compute step all arrive here, which is what makes the
//! policy check at this boundary (rule 44) worth having: there is no second
//! path around it.
//!
//! Models are held resident and are never swapped per call (rule 45), and a
//! model is identified by the SHA-256 of its GGUF file (rule 12) rather than
//! by a name someone could reuse for different weights.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod adapters;
pub mod domain;
pub mod error;
pub mod ports;

pub use adapters::CandleInferencer;
pub use domain::{ModelId, Pooling, Task};
pub use error::{Error, Result};
pub use ports::Inferencer;
