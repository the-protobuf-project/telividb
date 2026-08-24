//! A collection, as the catalogue holds it.

use crate::{Fingerprint, ResourceName, VectorFieldSpec};

/// One collection's identity and the vector fields it declares.
///
/// **Why a collection must exist before a point can be written to it.** A
/// vector field binds a width, a metric and a model identity (rule 12). Left
/// to be inferred from the first vector that happens to arrive, those become
/// whatever the first writer sent — so a second writer with a different model,
/// or a transposed vector of the right length, is silently accepted into the
/// same index. Declaring them up front turns that into a rejection at the
/// boundary instead of degraded recall nobody can attribute.
///
/// The descriptor set is stored beside this rather than inside it: it is bytes
/// the engine never parses, and it is far larger than everything here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    /// Resource name, e.g. `collections/documents`.
    pub name: ResourceName,

    /// Digest of the canonicalized descriptor set this was created from.
    ///
    /// Mirrored into every segment written under it, so a segment copied to
    /// another machine can be validated rather than merely read.
    pub fingerprint: Fingerprint,

    /// The named vector fields points in this collection may carry.
    ///
    /// A point writing to a field absent from this list is refused: silently
    /// creating it would defeat the declaration above.
    pub vector_fields: Vec<VectorFieldSpec>,
}

impl Collection {
    /// A collection with no vector fields yet.
    pub fn new(name: ResourceName, fingerprint: Fingerprint) -> Self {
        Self {
            name,
            fingerprint,
            vector_fields: Vec::new(),
        }
    }

    /// Declare a vector field.
    pub fn with_field(mut self, field: VectorFieldSpec) -> Self {
        self.vector_fields.push(field);
        self
    }

    /// Look up a declared field by name.
    pub fn field(&self, name: &str) -> Option<&VectorFieldSpec> {
        self.vector_fields.iter().find(|f| f.name == name)
    }
}
