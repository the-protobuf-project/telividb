//! Checking a write against what the collection declared.
//!
//! **Why this exists.** Without it, the first vector to arrive defines the
//! field: its width becomes the field's width and its model becomes the
//! field's provenance, implicitly. A second writer sending 512 floats where
//! the first sent 768 then creates a *second* field, or worse is accepted into
//! the first — and vectors from two models in one index degrade recall with
//! nothing anywhere reporting it (rule 12).
//!
//! Declaring up front turns all of that into a rejection at the boundary.

use super::point::PointsSvc;
use crate::error::storage_status;
use telividb_core::{Collection, ResourceName};
use tonic::Status;

impl PointsSvc {
    /// The declared collection, or a refusal naming what is missing.
    ///
    /// `Ok(None)` only when no catalogue is wired at all, which is the
    /// direct-construction path used by tests — a served process always has
    /// one.
    pub(super) fn declared(&self, collection: &ResourceName) -> Result<Option<Collection>, Status> {
        let Some(catalogue) = &self.catalogue else {
            return Ok(None);
        };

        match catalogue.get(collection).map_err(|e| storage_status(&e))? {
            Some(found) => Ok(Some(found)),
            None => Err(Status::not_found(format!(
                "collection {} does not exist; create it first, declaring the \
                 vector fields its points will carry",
                collection.as_str()
            ))),
        }
    }

    /// Refuse a vector that disagrees with its field's declaration.
    ///
    /// Checked before anything is written, so a rejected point leaves no
    /// partial state behind.
    pub(super) fn check_fields(
        declared: &Collection,
        vectors: &std::collections::BTreeMap<String, Vec<f32>>,
    ) -> Result<(), Status> {
        for (field, vector) in vectors {
            let Some(spec) = declared.field(field) else {
                let known: Vec<&str> = declared
                    .vector_fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect();
                return Err(Status::invalid_argument(format!(
                    "collection {} declares no vector field {field:?}; it has {known:?}",
                    declared.name.as_str()
                )));
            };

            if vector.len() != spec.dim.get() {
                return Err(Status::invalid_argument(format!(
                    "field {field:?} is {} dimensions wide; this vector has {}. \
                     A field's width is fixed at declaration, because stored \
                     vectors are read at that stride",
                    spec.dim.get(),
                    vector.len()
                )));
            }
        }
        Ok(())
    }
}
