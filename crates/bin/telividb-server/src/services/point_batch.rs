//! The AIP-233/234 batch methods, not yet implemented.
//!
//! Grouped here rather than left among the working handlers so `point.rs`
//! reads as what the service *does*. Batching is a later optimization pass:
//! the single-point path is correct, and a batch that merely looped it would
//! add API surface without adding throughput.

use telividb_proto::point::v1::{
    BatchCreatePointsRequest, BatchCreatePointsResponse, BatchDeletePointsRequest,
    BatchDeletePointsResponse, BatchGetPointsRequest, BatchGetPointsResponse,
};
use tonic::{Request, Response, Status};

/// Why every method here refuses.
const REASON: &str = "batch point operations are not yet implemented; \
                      use the single-point methods";

/// Create several points in one request.
pub(super) fn create(
    _request: Request<BatchCreatePointsRequest>,
) -> Result<Response<BatchCreatePointsResponse>, Status> {
    Err(Status::unimplemented(REASON))
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
