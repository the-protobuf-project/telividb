//! Domain types. Declarations and re-exports only.

mod content_ref;
mod fingerprint;
mod ids;
mod metric;
pub mod resource;
mod span;

pub use content_ref::ContentRef;
pub use fingerprint::Fingerprint;
pub use ids::{Dim, ExternalId, Ordinal};
pub use metric::Metric;
pub use resource::{ResourceName, Template};
pub use span::Span;
