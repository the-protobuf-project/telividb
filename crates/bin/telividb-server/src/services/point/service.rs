//! The Points service — create, get, list, delete.
//!
//! Batch operations return `Status::unimplemented` — batching is a later
//! optimization pass. Search lives in `point_search.rs`: it is the one handler
//! that composes several pieces rather than making a single store call.

use super::convert::to_wire;
use crate::services::vector::VectorFields;
use crate::error::to_status;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use telividb_buffers::protobuf::point::v1::points_server::Points;
use telividb_buffers::protobuf::point::v1::{
    BatchCreatePointsRequest, BatchCreatePointsResponse, BatchDeletePointsRequest,
    BatchDeletePointsResponse, BatchGetPointsRequest, BatchGetPointsResponse, CreatePointRequest,
    DeletePointRequest, GetPointRequest, ListPointsRequest, ListPointsResponse, Point,
    SearchPointsRequest, SearchPointsResponse,
};
use telividb_core::{PointStore, ResourceName};
use telividb_storage::RedbPointStore;
use telividb_telemetry::{fields, logger, redact};
use tonic::{Request, Response, Status};

/// Handles point create, get, list and delete, backed by one `redb` file per
/// collection under `data_dir`.
pub struct PointsSvc {
    pub(super) data_dir: PathBuf,
    /// One open `redb` handle per collection.
    ///
    /// redb takes an **exclusive file lock**, so two concurrent requests
    /// against the same collection cannot each open their own — the second
    /// fails with "Database already open". Caching makes concurrent access
    /// reuse one handle, and keeps a store open across requests rather than
    /// paying an open per call.
    pub(super) stores: Mutex<HashMap<String, Arc<RedbPointStore>>>,
    /// Vector fields, held across requests.
    ///
    /// Unlike every other store here, these cannot be opened per request: a
    /// field's unsealed buffer *is* its newest data, and dropping it between
    /// calls would discard every write since the last seal.
    pub(super) vectors: Arc<VectorFields>,
    /// The inference server, for requests that send text instead of vectors.
    ///
    /// Default-constructed means no model: such a request is refused with a
    /// message naming the flag that would enable it, rather than accepted and
    /// silently storing nothing.
    pub(super) embeddings: crate::services::vector::Embeddings,
    /// The collection catalogue, shared with the collection service.
    ///
    /// `None` only in tests that exercise the point path directly. In a served
    /// process it is always present, and a write to an undeclared collection
    /// is refused rather than creating one as a side effect.
    pub(super) catalogue: Option<Arc<telividb_storage::RedbCollectionStore>>,
}

impl PointsSvc {
    /// Serve points from underneath `data_dir`, opened lazily per request.
    ///
    /// Accepts no text until [`Self::with_embeddings`] supplies a model.
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            vectors: Arc::new(VectorFields::new(data_dir.clone())),
            stores: Mutex::new(HashMap::new()),
            data_dir,
            embeddings: crate::services::vector::Embeddings::default(),
            catalogue: None,
        }
    }

    /// Consult `catalogue` before accepting a write.
    pub fn with_catalogue(mut self, catalogue: Arc<telividb_storage::RedbCollectionStore>) -> Self {
        self.catalogue = Some(catalogue);
        self
    }

    /// Serve text-to-vector requests with `embeddings`.
    pub fn with_embeddings(mut self, embeddings: crate::services::vector::Embeddings) -> Self {
        self.embeddings = embeddings;
        self
    }
}

#[tonic::async_trait]
impl Points for PointsSvc {
    async fn create_point(
        &self,
        request: Request<CreatePointRequest>,
    ) -> Result<Response<Point>, Status> {
        super::create::create_point(self, request).await
    }

    async fn get_point(
        &self,
        request: Request<GetPointRequest>,
    ) -> Result<Response<Point>, Status> {
        let name = parse_name(&request.into_inner().name)?;
        let collection = parent_collection(&name)?;
        logger::debug!("get point").with_data(&serde_json::json!({
            fields::COLLECTION: redact::collection_label(collection.as_str()),
            fields::RESOURCE: redact::resource_token(name.as_str()),
        }));

        let store = self.store(&collection)?;
        match store.get(&name).map_err(|e| to_status(&e))? {
            Some(point) => Ok(Response::new(to_wire(point))),
            None => {
                // A miss is worth a record: "not found" is the answer most
                // often blamed on the wrong thing — a stale name, the wrong
                // collection, or a store that was never written to.
                logger::debug!("point not found").with_data(&serde_json::json!({
                    fields::RESOURCE: redact::resource_token(name.as_str()),
                }));
                Err(Status::not_found(format!("point {name} not found")))
            }
        }
    }

    async fn list_points(
        &self,
        request: Request<ListPointsRequest>,
    ) -> Result<Response<ListPointsResponse>, Status> {
        let parent = parse_name(&request.into_inner().parent)?;
        let store = self.store(&parent)?;
        let points = store.list(&parent).map_err(|e| to_status(&e))?;
        logger::debug!("list points").with_data(&serde_json::json!({
            fields::COLLECTION: redact::collection_label(parent.as_str()),
            fields::RESULTS_RETURNED: points.len(),
        }));
        Ok(Response::new(ListPointsResponse {
            points: points.into_iter().map(to_wire).collect(),
            next_page_token: String::new(),
        }))
    }

    async fn delete_point(
        &self,
        request: Request<DeletePointRequest>,
    ) -> Result<Response<Point>, Status> {
        super::delete::delete_point(self, request).await
    }

    async fn batch_create_points(
        &self,
        request: Request<BatchCreatePointsRequest>,
    ) -> Result<Response<BatchCreatePointsResponse>, Status> {
        super::batch::create(self, request).await
    }

    async fn batch_get_points(
        &self,
        request: Request<BatchGetPointsRequest>,
    ) -> Result<Response<BatchGetPointsResponse>, Status> {
        super::batch::get(request)
    }

    async fn batch_delete_points(
        &self,
        request: Request<BatchDeletePointsRequest>,
    ) -> Result<Response<BatchDeletePointsResponse>, Status> {
        super::batch::delete(request)
    }

    async fn search_points(
        &self,
        request: Request<SearchPointsRequest>,
    ) -> Result<Response<SearchPointsResponse>, Status> {
        super::search::search_points(self, request).await
    }
}

pub(super) fn parent_collection(name: &ResourceName) -> Result<ResourceName, Status> {
    name.parent()
        .ok_or_else(|| Status::invalid_argument(format!("{name} has no parent collection")))
}

pub(super) fn parse_name(raw: &str) -> Result<ResourceName, Status> {
    ResourceName::parse(raw).map_err(|e| Status::invalid_argument(e.to_string()))
}
