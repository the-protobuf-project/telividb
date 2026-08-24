//! The Points service — create, get, list, delete.
//!
//! Batch operations return `Status::unimplemented` — batching is a later
//! optimization pass. Search lives in `point_search.rs`: it is the one handler
//! that composes several pieces rather than making a single store call.

use super::point_convert::to_wire;
use super::vectors::VectorFields;
use crate::error::{storage_status, to_status};
use std::path::PathBuf;
use telividb_core::{PointStore, ResourceName};
use telividb_proto::point::v1::points_server::Points;
use telividb_proto::point::v1::{
    BatchCreatePointsRequest, BatchCreatePointsResponse, BatchDeletePointsRequest,
    BatchDeletePointsResponse, BatchGetPointsRequest, BatchGetPointsResponse, CreatePointRequest,
    DeletePointRequest, GetPointRequest, ListPointsRequest, ListPointsResponse, Point,
    SearchPointsRequest, SearchPointsResponse,
};
use telividb_storage::{PointStoreConfig, RedbPointStore, open_point_store};
use telividb_telemetry::{fields, logger, redact};
use tonic::{Request, Response, Status};

/// Handles point create, get, list and delete, backed by one `redb` file per
/// collection under `data_dir`.
pub struct PointsSvc {
    data_dir: PathBuf,
    /// Vector fields, held across requests.
    ///
    /// Unlike every other store here, these cannot be opened per request: a
    /// field's unsealed buffer *is* its newest data, and dropping it between
    /// calls would discard every write since the last seal.
    pub(super) vectors: VectorFields,
}

impl PointsSvc {
    /// Serve points from underneath `data_dir`, opened lazily per request.
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            vectors: VectorFields::new(data_dir.clone()),
            data_dir,
        }
    }

    /// Path to the `redb` file for `collection`, e.g. `media` from
    /// `collections/media`.
    fn store_path(&self, collection: &ResourceName) -> PathBuf {
        self.data_dir.join(collection.leaf()).join("points.redb")
    }

    fn open(&self, collection: &ResourceName) -> Result<Box<dyn PointStore>, Status> {
        open_point_store(&PointStoreConfig::Redb {
            path: self.store_path(collection),
        })
        .map_err(|e| storage_status(&e))
    }

    pub(super) fn open_writer(&self, collection: &ResourceName) -> Result<RedbPointStore, Status> {
        RedbPointStore::open(&self.store_path(collection)).map_err(|e| storage_status(&e))
    }
}

pub(super) fn parse_name(raw: &str) -> Result<ResourceName, Status> {
    ResourceName::parse(raw).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl Points for PointsSvc {
    async fn create_point(
        &self,
        request: Request<CreatePointRequest>,
    ) -> Result<Response<Point>, Status> {
        super::point_create::create_point(self, request).await
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

        let store = self.open(&collection)?;
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
        let store = self.open(&parent)?;
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
    ) -> Result<Response<()>, Status> {
        let name = parse_name(&request.into_inner().name)?;
        let collection = parent_collection(&name)?;
        // A mutation is worth one record per call at info: they are rare and
        // they are what an incident gets reconstructed from.
        logger::info!("delete point").with_data(&serde_json::json!({
            fields::COLLECTION: redact::collection_label(collection.as_str()),
            fields::RESOURCE: redact::resource_token(name.as_str()),
        }));

        let store = self.open_writer(&collection)?;
        if store.delete(&name).map_err(|e| storage_status(&e))? {
            Ok(Response::new(()))
        } else {
            logger::debug!("delete missed: no such point").with_data(&serde_json::json!({
                fields::RESOURCE: redact::resource_token(name.as_str()),
            }));
            Err(Status::not_found(format!("point {name} not found")))
        }
    }

    async fn batch_create_points(
        &self,
        request: Request<BatchCreatePointsRequest>,
    ) -> Result<Response<BatchCreatePointsResponse>, Status> {
        super::point_batch::create(request)
    }

    async fn batch_get_points(
        &self,
        request: Request<BatchGetPointsRequest>,
    ) -> Result<Response<BatchGetPointsResponse>, Status> {
        super::point_batch::get(request)
    }

    async fn batch_delete_points(
        &self,
        request: Request<BatchDeletePointsRequest>,
    ) -> Result<Response<BatchDeletePointsResponse>, Status> {
        super::point_batch::delete(request)
    }

    async fn search_points(
        &self,
        request: Request<SearchPointsRequest>,
    ) -> Result<Response<SearchPointsResponse>, Status> {
        super::point_search::search_points(self, request).await
    }
}

fn parent_collection(name: &ResourceName) -> Result<ResourceName, Status> {
    name.parent()
        .ok_or_else(|| Status::invalid_argument(format!("{name} has no parent collection")))
}
