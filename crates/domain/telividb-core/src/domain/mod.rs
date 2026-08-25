//! Domain types. Declarations and re-exports only.

mod collection;
mod content_ref;
mod edge;
mod fingerprint;
mod ids;
mod metric;
mod point;
pub mod resource;
mod span;

pub use collection::Collection;
pub use content_ref::ContentRef;
pub use edge::Edge;
pub use fingerprint::Fingerprint;
pub use ids::{Dim, ExternalId, Ordinal};
pub use metric::Metric;
pub use point::Point;
pub use resource::{ResourceName, Template};
pub use span::Span;
