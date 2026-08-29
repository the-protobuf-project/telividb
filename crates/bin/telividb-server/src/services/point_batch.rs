//! The AIP-233/234 batch methods.
//!
//! # What batching buys here, and what it does not
//!
//! Not storage throughput. Each point still takes the same path as a single
//! `CreatePoint`: vectors to the field's WAL, the row binding and the point to
//! `redb`. A genuinely batched write — one WAL append and one `redb`
//! transaction for the whole request — is a later optimization, and it belongs
//! in the storage layer rather than here.
//!
//! What it buys is the round trip. Importing ten thousand rows one at a time is
//! ten thousand request/response pairs before any work happens; as a batch it
//! is one. For a desktop app importing a CSV that is the whole cost, which is
//! why this is worth implementing before the storage-level batching that would
//! make it faster still.
//!
//! # Atomicity
//!
//! AIP-233 asks a batch create to be atomic, and this one is not: there is no
//! transaction spanning the WAL and `redb`, so a failure partway leaves the
//! points written before it. Rather than claim otherwise, the failure names how
//! far it got — see [`create`].

use super::point::PointsSvc;
use telividb_buffers::protobuf::point::v1::{
    BatchCreatePointsRequest, BatchCreatePointsResponse, BatchDeletePointsRequest,
    BatchDeletePointsResponse, BatchGetPointsRequest, BatchGetPointsResponse,
};
use telividb_telemetry::{fields, logger, redact};
use tonic::{Request, Response, Status};

/// Why the methods that remain unimplemented refuse.
const REASON: &str = "batch point operations are not yet implemented; \
                      use the single-point methods";

/// Create several points in one request.
///
/// Stops at the first failure and reports which item it was, because a caller
/// that knows the index can fix that row and resubmit from there. Continuing
/// past a failure would need a per-item status in the response, which the
/// message does not carry — inventing one here would be a wire change made in
/// an implementation.
pub(super) async fn create(
    svc: &PointsSvc,
    request: Request<BatchCreatePointsRequest>,
) -> Result<Response<BatchCreatePointsResponse>, Status> {
    let req = request.into_inner();
    if req.requests.is_empty() {
        return Err(Status::invalid_argument(
            "requests must not be empty: a batch with nothing in it is a \
             mistake rather than a no-op",
        ));
    }

    logger::info!("batch create points").with_data(&serde_json::json!({
        fields::COLLECTION: redact::collection_label(&req.parent),
        "telividb.batch.size": req.requests.len(),
    }));

    let mut points = Vec::with_capacity(req.requests.len());
    for (index, mut item) in req.requests.into_iter().enumerate() {
        // The batch's `parent` is authoritative. An item carrying a different
        // one would write outside the collection the caller named, so the
        // batch's wins and a mismatch is refused rather than silently
        // corrected.
        if !item.parent.is_empty() && item.parent != req.parent {
            return Err(Status::invalid_argument(format!(
                "requests[{index}].parent is {:?}, but the batch names {:?}; \
                 a batch writes to one collection",
                item.parent, req.parent
            )));
        }
        item.parent = req.parent.clone();

        let created = super::point_create::create_point(svc, Request::new(item))
            .await
            .map_err(|status| annotate(index, points.len(), status))?;
        points.push(created.into_inner());
    }

    Ok(Response::new(BatchCreatePointsResponse { points }))
}

/// Say which item failed, and how many were already written.
///
/// Without the count a caller cannot resubmit: retrying the whole batch would
/// duplicate everything before the failure, and skipping it entirely would drop
/// the rest.
fn annotate(index: usize, written: usize, status: Status) -> Status {
    Status::new(
        status.code(),
        format!(
            "requests[{index}] failed after {written} point(s) were already \
             written: {}",
            status.message()
        ),
    )
}

/// Retrieve several points in one request.
pub(super) fn get(
    _request: Request<BatchGetPointsRequest>,
) -> Result<Response<BatchGetPointsResponse>, Status> {
    Err(Status::unimplemented(REASON))
}

/// Delete several points in one request.
pub(super) fn delete(
    _request: Request<BatchDeletePointsRequest>,
) -> Result<Response<BatchDeletePointsResponse>, Status> {
    Err(Status::unimplemented(REASON))
}
