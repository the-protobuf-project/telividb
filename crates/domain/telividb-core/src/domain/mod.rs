//! Domain types. Declarations and re-exports only.

mod architecture;
mod collection;
mod content_ref;
mod edge;
mod fingerprint;
mod identity;
mod ids;
mod metric;
mod modality;
mod point;
pub mod resource;
mod span;
mod tenancy;

pub use architecture::Architecture;
pub use collection::Collection;
pub use content_ref::ContentRef;
pub use edge::Edge;
pub use fingerprint::Fingerprint;
pub use identity::{RoleBinding, User, UserGroup};
pub use ids::{Dim, ExternalId, Ordinal};
pub use metric::Metric;
pub use modality::Modality;
pub use point::Point;
pub use resource::{ResourceName, Template};
pub use span::Span;
pub use tenancy::{Lifecycle, Organization, Project, Protection, Session, Space};
