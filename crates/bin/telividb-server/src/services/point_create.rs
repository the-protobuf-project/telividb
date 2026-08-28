//! The `CreatePoint` handler.
//!
//! Split from `point.rs` because it is the only write path that touches more
//! than one store: vectors go to the field's WAL, the row-to-name binding to
//! `redb`, and the point itself to `redb` — in that order, and the order is
//! the interesting part. See the comment inside.

use super::point::{PointsSvc, parse_name};
use super::point_convert::{to_domain, to_wire};
use super::vectors::VectorFields;
use crate::error::{storage_status, to_status};
use std::sync::Arc;
use telividb_buffers::protobuf::point::v1::{CreatePointRequest, Point};
use telividb_core::ResourceName;
use telividb_storage::RedbPointStore;
use telividb_telemetry::{fields, logger, redact};
use tonic::{Request, Response, Status};

/// Create one point, persisting its vectors before the point itself.
pub(super) async fn create_point(
    svc: &PointsSvc,
    request: Request<CreatePointRequest>,
) -> Result<Response<Point>, Status> {
    let req = request.into_inner();
    if req.point_id.is_empty() {
        return Err(Status::invalid_argument(
            "point_id must not be empty: it forms the final path segment \
             of the point's resource name",
        ));
    }
    let parent = parse_name(&req.parent)?;
    let name = parse_name(&format!("{}/points/{}", parent.as_str(), req.point_id))?;

    logger::info!("create point").with_data(&serde_json::json!({
        fields::COLLECTION: redact::collection_label(parent.as_str()),
        fields::RESOURCE: redact::resource_token(name.as_str()),
    }));

    // Text becomes a vector *before* the domain conversion, so everything
    // downstream sees a point that carries vectors and nothing has to know
    // that some of them started as text.
    let mut wire = req.point.unwrap_or_default();
    svc.embeddings.resolve_point(&mut wire).await?;
    let point = to_domain(name, wire)?;

    // Checked before any write: a rejected point must leave no partial state.
    if let Some(declared) = svc.declared(&parent)? {
        PointsSvc::check_fields(&declared, &point.vectors)?;
    }

    // Storage is synchronous: redb commits, WAL fsyncs and mmap'd segment
    // reads all block. Running them on a tonic executor thread would stall
    // every other request sharing it, which invariant 5 forbids — so the whole
    // write happens on the blocking pool.
    let vectors = Arc::clone(&svc.vectors);
    let store = svc.store(&parent)?;
    let collection = parent.clone();
    let point =
        tokio::task::spawn_blocking(move || write_point(&vectors, &store, &collection, point))
            .await
            .map_err(|e| Status::internal(format!("create task failed: {e}")))??;

    Ok(Response::new(to_wire(point)))
}

/// The blocking half of a create: append vectors, then write the point and its
/// row bindings in one transaction.
///
/// **Ordering matters.** The duplicate check happens *first*, before any vector
/// is appended, so a rejected create leaves no orphan rows in the field's WAL.
/// The bindings then land in the same transaction as the point itself, so a
/// crash cannot leave a row pointing at a resource that was never created.
fn write_point(
    vectors: &VectorFields,
    store: &RedbPointStore,
    collection: &ResourceName,
    point: telividb_core::Point,
) -> Result<telividb_core::Point, Status> {
    if store.exists(&point.name).map_err(|e| storage_status(&e))? {
        logger::debug!("create refused: point already exists").with_data(&serde_json::json!({
            fields::RESOURCE: redact::resource_token(point.name.as_str()),
        }));
        return Err(Status::already_exists(format!(
            "point {} already exists",
            point.name
        )));
    }

    let mut rows = Vec::with_capacity(point.vectors.len());
    for (field, vector) in &point.vectors {
        let row = vectors
            .append(collection, field, vector)
            .map_err(|e| to_status(&e))?;
        rows.push((field.clone(), row as u64));
        logger::debug!("vector appended").with_data(&serde_json::json!({
            fields::COLLECTION: redact::collection_label(collection.as_str()),
            fields::FIELD: field,
            fields::DIM: vector.len(),
        }));
    }

    if !store
        .create_with_rows(&point, &rows)
        .map_err(|e| storage_status(&e))?
    {
        // Lost a race with a concurrent create of the same name.
        return Err(Status::already_exists(format!(
            "point {} already exists",
            point.name
        )));
    }
    Ok(point)
}
