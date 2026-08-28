//! The `DeletePoint` handler.
//!
//! Split from `point.rs` because deletion here is not the one-line operation
//! its signature suggests. It reads before it writes, and it returns the point
//! it removed rather than an empty message — both consequences of the same
//! fact, that a delete in this engine is deferred rather than immediate.

use super::point::{PointsSvc, parent_collection, parse_name};
use super::point_convert::to_wire;
use crate::error::{storage_status, to_status};
use telividb_buffers::protobuf::point::v1::{DeletePointRequest, Point};
use telividb_core::PointStore;
use telividb_telemetry::{fields, logger, redact};
use tonic::{Request, Response, Status};

/// Tombstone a point and return the resource that was removed.
///
/// Sealed segments are immutable, so nothing is erased here: the point is
/// tombstoned, queries exclude it immediately, and compaction reclaims the
/// bytes later. That is why the response carries the point rather than nothing
/// — AIP-135's soft-delete form — and why a caller who needs erasure rather
/// than concealment has to force compaction and confirm it finished.
pub(super) async fn delete_point(
    svc: &PointsSvc,
    request: Request<DeletePointRequest>,
) -> Result<Response<Point>, Status> {
    let name = parse_name(&request.into_inner().name)?;
    let collection = parent_collection(&name)?;

    // A mutation is worth one record per call at info: they are rare, and they
    // are what an incident gets reconstructed from.
    logger::info!("delete point").with_data(&serde_json::json!({
        fields::COLLECTION: redact::collection_label(collection.as_str()),
        fields::RESOURCE: redact::resource_token(name.as_str()),
    }));

    let store = svc.store(&collection)?;

    // Read first. After the tombstone there is nothing left to describe, and
    // the response has to carry what was removed.
    let deleted = store.get(&name).map_err(|e| to_status(&e))?.map(to_wire);

    if store.delete(&name).map_err(|e| storage_status(&e))? {
        Ok(Response::new(deleted.unwrap_or_default()))
    } else {
        logger::debug!("delete missed: no such point").with_data(&serde_json::json!({
            fields::RESOURCE: redact::resource_token(name.as_str()),
        }));
        Err(Status::not_found(format!("point {name} not found")))
    }
}
