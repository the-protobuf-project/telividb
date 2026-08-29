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
mod tenancy;

pub use collection::Collection;
pub use content_ref::ContentRef;
pub use edge::Edge;
pub use fingerprint::Fingerprint;
pub use ids::{Dim, ExternalId, Ordinal};
pub use metric::Metric;
pub use point::Point;
pub use resource::{ResourceName, Template};
pub use span::Span;
pub use tenancy::{Lifecycle, Organization, Project, Protection, Session, Space};
