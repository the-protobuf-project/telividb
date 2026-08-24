//! The `CreatePoint` handler.
//!
//! Split from `point.rs` because it is the only write path that touches more
//! than one store: vectors go to the field's WAL, the row-to-name binding to
//! `redb`, and the point itself to `redb` — in that order, and the order is
//! the interesting part. See the comment inside.

use super::point::{PointsSvc, parse_name};
use super::point_convert::{to_domain, to_wire};
use crate::error::{storage_status, to_status};
use telividb_proto::point::v1::{CreatePointRequest, Point};
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

    let point = to_domain(name, req.point.unwrap_or_default())?;
    let store = svc.open_writer(&parent)?;

    // Vectors first. A vector written for a point that then fails to
    // create leaves an orphan row — wasteful but harmless, since nothing
    // binds it to a name. The reverse order would let a point exist whose
    // vectors were never stored, which search would silently miss.
    for (field, vector) in &point.vectors {
        let row = svc
            .vectors
            .append(&parent, field, vector)
            .map_err(|e| to_status(&e))?;
        store
            .bind_row(field, row as u64, &point.name)
            .map_err(|e| storage_status(&e))?;
        logger::debug!("vector appended").with_data(&serde_json::json!({
            fields::COLLECTION: redact::collection_label(parent.as_str()),
            fields::FIELD: field,
            fields::DIM: vector.len(),
        }));
    }

    if !store.create(&point).map_err(|e| storage_status(&e))? {
        logger::debug!("create refused: point already exists").with_data(&serde_json::json!({
            fields::RESOURCE: redact::resource_token(point.name.as_str()),
        }));
        return Err(Status::already_exists(format!(
            "point {} already exists",
            point.name
        )));
    }
    Ok(Response::new(to_wire(point)))
}
