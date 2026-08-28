//! The `:search` custom method.
//!
//! Split from `point.rs` because it is the one handler that is not CRUD: it
//! composes the vector state, the index and the row-to-name mapping, where the
//! others are a single store call each.

use super::point::{PointsSvc, parse_name};
use super::point_convert::{to_wire, vector_to_domain};
use crate::error::{storage_status, to_status};
use std::sync::Arc;
use telividb_buffers::protobuf::point::v1::{
    SearchPointsRequest, SearchPointsResponse, SearchResult,
};
use telividb_core::PointStore;
use telividb_telemetry::{fields, logger, redact};
use tonic::{Request, Response, Status};

/// Neighbours returned when a caller does not say.
const DEFAULT_K: usize = 10;

/// The most a caller may ask for.
///
/// `k` drives how much the index scores and how much is allocated, so leaving
/// it unbounded lets one request ask the server to do arbitrary work.
const MAX_K: usize = 1_000;

/// Resolve the request's query to a vector, embedding text if that is what it
/// carries.
///
/// Exactly one of `query` and `query_text` must be set. Both is ambiguous and
/// neither is not a query — refusing beats silently preferring one, which
/// would search for something the caller did not ask for.
async fn resolve_query(svc: &PointsSvc, req: &SearchPointsRequest) -> Result<Vec<f32>, Status> {
    match (req.query.as_ref(), req.query_text.is_empty()) {
        (Some(_), false) => Err(Status::invalid_argument(
            "set exactly one of query and query_text; there is no correct way \
             to choose between them",
        )),
        (Some(vector), true) => vector_to_domain(vector),
        (None, false) => svc.embeddings.embed_query(&req.query_text).await,
        (None, true) => Err(Status::invalid_argument(
            "a query is required: send either query (a vector) or query_text",
        )),
    }
}

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
    let query = resolve_query(svc, &req).await?;
    // Clamped: `k` sizes the work the index does and the buffer it fills, so
    // an unbounded page_size is a request to allocate arbitrarily.
    let k = if req.page_size <= 0 {
        DEFAULT_K
    } else {
        (req.page_size as usize).min(MAX_K)
    };

    // Storage is synchronous: mmap'd segment reads fault, redb reads block,
    // and a GPU upload is not instant. Invariant 5 forbids doing any of that
    // on a tonic executor thread, so the whole read runs on the blocking pool.
    let query_dim = query.len();
    let vectors = Arc::clone(&svc.vectors);
    let store = svc.store(&parent)?;
    let field_id = req.field_id.clone();
    let collection = parent.clone();
    let results = tokio::task::spawn_blocking(move || {
        let hits = vectors
            .search(&collection, &field_id, &query, k)
            .map_err(|e| to_status(&e))?;

        // Rows are internal; only resource names cross the boundary
        // (invariant 9). A row with no binding is skipped rather than
        // reported: it is a vector whose point never finished creating.
        let mut results = Vec::with_capacity(hits.len());
        for (row, score) in hits {
            let Some(name) = store
                .row_owner(&field_id, row as u64)
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
        Ok::<_, Status>(results)
    })
    .await
    .map_err(|e| Status::internal(format!("search task failed: {e}")))??;

    logger::debug!("search points").with_data(&serde_json::json!({
        fields::COLLECTION: redact::collection_label(parent.as_str()),
        fields::FIELD: req.field_id,
        fields::K: k,
        fields::DIM: query_dim,
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
