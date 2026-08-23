//! Domain vocabulary for episteme. No I/O lives here — see ARCHITECTURE.md §2.
#![forbid(unsafe_code)]

pub mod domain;
pub mod error;
pub mod ports;
pub mod schema;

pub use domain::{
    ContentRef, Dim, ExternalId, Fingerprint, Metric, Ordinal, ResourceName, Span, Template,
};
pub use error::{Error, Result};
pub use ports::{SchemaReader, VectorStore};
pub use schema::{CollectionSchema, Compatibility, IndexKind, PointType, VectorFieldSpec};
