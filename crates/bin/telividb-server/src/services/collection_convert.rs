//! Converting between the wire `Collection` and the domain one.

use telividb_core::{
    Collection, Dim, Fingerprint, IndexKind, Metric, ResourceName, VectorFieldSpec,
};
use telividb_proto::collection::v1 as wire;
use tonic::Status;

/// Build a domain collection from a create request's payload.
pub(super) fn to_domain(
    name: ResourceName,
    fingerprint: Fingerprint,
    wire: &wire::Collection,
) -> Result<Collection, Status> {
    let mut collection = Collection::new(name, fingerprint);

    for field in &wire.vector_fields {
        if field.field_id.is_empty() {
            return Err(Status::invalid_argument(
                "a vector field needs a field_id: it is how a point names \
                 which field a vector belongs to",
            ));
        }
        let dim = Dim::new(field.dimensions.max(0) as u32).map_err(|_| {
            Status::invalid_argument(format!(
                "vector field {:?} declares {} dimensions; a width must be \
                 positive, and it is fixed for the life of the field",
                field.field_id, field.dimensions
            ))
        })?;

        // A repeat would silently shadow the earlier declaration, so a caller
        // would believe a width or metric that is not in force.
        if collection.field(&field.field_id).is_some() {
            return Err(Status::invalid_argument(format!(
                "vector field {:?} is declared more than once",
                field.field_id
            )));
        }

        collection.vector_fields.push(VectorFieldSpec {
            name: field.field_id.clone(),
            dim,
            metric: metric_to_domain(field.metric)?,
            index: index_to_domain(field.index_kind),
            model: field.model.clone(),
            model_fingerprint: Fingerprint::unset(),
            query_encoder: non_empty(&field.query_encoder),
            permission: non_empty(&field.permission),
        });
    }

    Ok(collection)
}

/// Render a domain collection for the wire.
///
/// Counts are left at zero: they are `OUTPUT_ONLY` statistics the catalogue
/// does not maintain yet, and inventing a number would be worse than an
/// obvious placeholder.
pub(super) fn to_wire(collection: &Collection, descriptor_set: Vec<u8>) -> wire::Collection {
    wire::Collection {
        name: collection.name.as_str().to_owned(),
        descriptor_set: descriptor_set.into(),
        schema_fingerprint: collection.fingerprint.as_bytes().to_vec().into(),
        vector_fields: collection
            .vector_fields
            .iter()
            .map(|field| wire::VectorField {
                field_id: field.name.clone(),
                dimensions: field.dim.get() as i32,
                metric: metric_to_wire(field.metric),
                index_kind: index_to_wire(field.index),
                codec: wire::Codec::Unspecified as i32,
                model: field.model.clone(),
                query_encoder: field.query_encoder.clone().unwrap_or_default(),
                permission: field.permission.clone().unwrap_or_default(),
            })
            .collect(),
        live_point_count: 0,
        tombstoned_point_count: 0,
        segment_count: 0,
    }
}

/// An unset proto3 string is empty, which is not the same as a declared one.
fn non_empty(value: &str) -> Option<String> {
    match value.is_empty() {
        true => None,
        false => Some(value.to_owned()),
    }
}

/// Read the metric, refusing `UNSPECIFIED`.
///
/// Not defaulted: the metric decides what "nearest" means, and a field created
/// under the wrong one ranks correctly-stored vectors wrongly, without error.
fn metric_to_domain(value: i32) -> Result<Metric, Status> {
    match wire::Metric::try_from(value) {
        Ok(wire::Metric::Dot) => Ok(Metric::Dot),
        Ok(wire::Metric::L2) => Ok(Metric::L2),
        Ok(wire::Metric::Cosine) => Ok(Metric::Cosine),
        _ => Err(Status::invalid_argument(
            "each vector field must declare a metric: it decides what nearest \
             means, and the wrong one ranks stored vectors wrongly with no error",
        )),
    }
}

/// Render the metric for the wire.
fn metric_to_wire(metric: Metric) -> i32 {
    let value = match metric {
        Metric::Dot => wire::Metric::Dot,
        Metric::L2 => wire::Metric::L2,
        Metric::Cosine => wire::Metric::Cosine,
    };
    value as i32
}

/// Read the index kind, defaulting to flat.
///
/// Safe to default where the metric is not: flat is exhaustive, so an
/// unspecified index gives exact results and only costs speed.
fn index_to_domain(value: i32) -> IndexKind {
    match wire::IndexKind::try_from(value) {
        Ok(wire::IndexKind::Hnsw) => IndexKind::Hnsw,
        Ok(wire::IndexKind::IvfPq) => IndexKind::IvfPq,
        _ => IndexKind::Flat,
    }
}

/// Render the index kind for the wire.
fn index_to_wire(index: IndexKind) -> i32 {
    let value = match index {
        IndexKind::Flat => wire::IndexKind::Flat,
        IndexKind::Hnsw => wire::IndexKind::Hnsw,
        IndexKind::IvfPq => wire::IndexKind::IvfPq,
    };
    value as i32
}
