//! The `:search` custom method.
//!
//! Split from `point.rs` because it is the one handler that is not CRUD: it
//! composes the vector state, the index and the row-to-name mapping, where the
//! others are a single store call each.

use super::point::{PointsSvc, parse_name};
use super::point_convert::{to_wire, vector_to_domain};
use crate::error::{storage_status, to_status};
use telividb_core::PointStore;
use telividb_proto::point::v1::{SearchPointsRequest, SearchPointsResponse, SearchResult};
use telividb_telemetry::{fields, logger, redact};
use tonic::{Request, Response, Status};

/// Neighbours returned when a caller does not say.
const DEFAULT_K: usize = 10;

/// Nearest-neighbour search over one named vector field.
pub(super) async fn search_points(
    svc: &PointsSvc,
    request: Request<SearchPointsRequest>,
) -> Result<Response<SearchPointsResponse>, Status> {
    let req = request.into_inner();
    let parent = parse_name(&req.parent)?;
    if req.field_id.is_empty() {
        return Err(Status::invalid_argument(
            "field_id is required: each vector field has its own model and \
             metric, so a query is meaningful only against one of them",
        ));
    }
    let query = req
        .query
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("query vector is required"))?;
    let query = vector_to_domain(query)?;
    let k = if req.page_size <= 0 {
        DEFAULT_K
    } else {
        req.page_size as usize
    };

    let hits = svc
        .vectors
        .search(&parent, &req.field_id, &query, k)
        .map_err(|e| to_status(&e))?;

    // Rows are internal; only resource names cross the boundary
    // (invariant 9). A row with no binding is skipped rather than
    // reported: it is a vector whose point never finished creating.
    //
    // One handle, not two: redb takes an exclusive file lock, so opening
    // the same store through both the port and the concrete adapter would
    // deadlock against itself. `RedbPointStore` implements `PointStore`,
    // so the single writer handle serves both the row lookup and the read.
    let store = svc.open_writer(&parent)?;
    let mut results = Vec::with_capacity(hits.len());
    for (row, score) in hits {
        let Some(name) = store
            .row_owner(&req.field_id, row as u64)
            .map_err(|e| storage_status(&e))?
        else {
            continue;
        };
        let Some(point) = store.get(&name).map_err(|e| to_status(&e))? else {
            continue;
        };
        results.push(SearchResult {
            point: Some(to_wire(point)),
            score,
        });
    }

    logger::debug!("search points").with_data(&serde_json::json!({
        fields::COLLECTION: redact::collection_label(parent.as_str()),
        fields::FIELD: req.field_id,
        fields::K: k,
        fields::QUERY: redact::vector_shape(&query),
        fields::RESULTS_RETURNED: results.len(),
    }));

    Ok(Response::new(SearchPointsResponse {
        results,
        next_page_token: String::new(),
        // Single-node: every source answered, unconditionally. The field
        // exists from day one because adding it later would break the
        // most-used message in the API.
        complete: true,
        answered_source_count: 1,
        total_source_count: 1,
        locked_vaults: Vec::new(),
        stats: None,
    }))
}
