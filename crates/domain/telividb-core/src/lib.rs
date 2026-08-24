//! Domain vocabulary for telividb. No I/O lives here — see ARCHITECTURE.md §2.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod domain;
pub mod error;
pub mod ports;
pub mod schema;

pub use domain::{
    Collection, ContentRef, Dim, Edge, ExternalId, Fingerprint, Metric, Ordinal, Point,
    ResourceName, Span, Template,
};
pub use error::{Error, Result};
pub use ports::{
    GraphStore, PointStore, PreparedQuery, PreparedState, ScanTier, SchemaReader, VectorStore,
};
pub use schema::{CollectionSchema, Compatibility, IndexKind, PointType, VectorFieldSpec};
