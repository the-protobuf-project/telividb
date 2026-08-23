//! A collection and the point types it holds.

use super::VectorFieldSpec;
use crate::Fingerprint;

/// One AIP resource, projected as a node type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointType {
    /// Resource type, e.g. `media.episteme.dev/Shot`.
    pub type_name: String,
    /// Resource name pattern, e.g. `recordings/{recording}/shots/{shot}`.
    pub pattern: String,
    /// Named vector fields declared on this point type.
    pub vector_fields: Vec<VectorFieldSpec>,
    /// Edge type names declared by `resource_reference` fields.
    pub edges: Vec<String>,
    /// Whether points of this type carry a temporal span.
    pub has_span: bool,
}

impl PointType {
    /// Look up one named vector field by name.
    pub fn vector_field(&self, name: &str) -> Option<&VectorFieldSpec> {
        self.vector_fields.iter().find(|f| f.name == name)
    }
}

/// The resolved schema of one collection.
///
/// Authoritative form is the `FileDescriptorSet` in `meta.redb`; this is what a
/// `SchemaReader` resolves it into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSchema {
    /// Collection name, matching the database name in the schema.
    pub collection: String,
    /// Point types this collection holds.
    pub point_types: Vec<PointType>,
    /// Digest of the canonicalized descriptor set this was resolved from.
    pub fingerprint: Fingerprint,
}

impl CollectionSchema {
    /// An empty schema carrying only its identity and fingerprint.
    pub fn new(collection: impl Into<String>, fingerprint: Fingerprint) -> Self {
        Self {
            collection: collection.into(),
            point_types: Vec::new(),
            fingerprint,
        }
    }

    /// Add a point type, returning the extended schema.
    pub fn with_point_type(mut self, point_type: PointType) -> Self {
        self.point_types.push(point_type);
        self
    }

    /// Look up one point type by its resource type name.
    pub fn point_type(&self, type_name: &str) -> Option<&PointType> {
        self.point_types.iter().find(|p| p.type_name == type_name)
    }

    /// Every vector field across every point type, paired with its owner.
    pub fn vector_fields(&self) -> impl Iterator<Item = (&PointType, &VectorFieldSpec)> {
        self.point_types
            .iter()
            .flat_map(|p| p.vector_fields.iter().map(move |f| (p, f)))
    }
}
