//! The Points service — create, get, list, delete.
//!
//! Batch operations and search return `Status::unimplemented`: batching is a
//! later optimization pass, and search is the vector service's job, which
//! needs the named-vector fields this slice deliberately leaves out.

use super::point_convert::{to_domain, to_wire};
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
}

impl PointsSvc {
    /// Serve points from underneath `data_dir`, opened lazily per request.
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
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

    fn open_writer(&self, collection: &ResourceName) -> Result<RedbPointStore, Status> {
        RedbPointStore::open(&self.store_path(collection)).map_err(|e| storage_status(&e))
    }
}

fn parse_name(raw: &str) -> Result<ResourceName, Status> {
    ResourceName::parse(raw).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl Points for PointsSvc {
    async fn create_point(
        &self,
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
            fields::RESOURCE: redact::resource_token(name.as_str()),
        }));

        let point = to_domain(name, req.point.unwrap_or_default())?;
        let store = self.open_writer(&parent)?;
        if !store.create(&point).map_err(|e| storage_status(&e))? {
            return Err(Status::already_exists(format!(
                "point {} already exists",
                point.name
            )));
        }
        Ok(Response::new(to_wire(point)))
    }

    async fn get_point(
        &self,
        request: Request<GetPointRequest>,
    ) -> Result<Response<Point>, Status> {
        let name = parse_name(&request.into_inner().name)?;
        let collection = parent_collection(&name)?;
        let store = self.open(&collection)?;
        match store.get(&name).map_err(|e| to_status(&e))? {
            Some(point) => Ok(Response::new(to_wire(point))),
            None => Err(Status::not_found(format!("point {name} not found"))),
        }
    }

    async fn list_points(
        &self,
        request: Request<ListPointsRequest>,
    ) -> Result<Response<ListPointsResponse>, Status> {
        let parent = parse_name(&request.into_inner().parent)?;
        let store = self.open(&parent)?;
        let points = store.list(&parent).map_err(|e| to_status(&e))?;
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
        let store = self.open_writer(&collection)?;
        if store.delete(&name).map_err(|e| storage_status(&e))? {
            Ok(Response::new(()))
        } else {
            Err(Status::not_found(format!("point {name} not found")))
        }
    }

    async fn batch_create_points(
        &self,
        _request: Request<BatchCreatePointsRequest>,
    ) -> Result<Response<BatchCreatePointsResponse>, Status> {
        Err(Status::unimplemented(
            "batch point operations are not yet implemented",
        ))
    }

    async fn batch_get_points(
        &self,
        _request: Request<BatchGetPointsRequest>,
    ) -> Result<Response<BatchGetPointsResponse>, Status> {
        Err(Status::unimplemented(
            "batch point operations are not yet implemented",
        ))
    }

    async fn batch_delete_points(
        &self,
        _request: Request<BatchDeletePointsRequest>,
    ) -> Result<Response<BatchDeletePointsResponse>, Status> {
        Err(Status::unimplemented(
            "batch point operations are not yet implemented",
        ))
    }

    async fn search_points(
        &self,
        _request: Request<SearchPointsRequest>,
    ) -> Result<Response<SearchPointsResponse>, Status> {
        Err(Status::unimplemented(
            "search is not yet implemented; it lands with the vector service",
        ))
    }
}

fn parent_collection(name: &ResourceName) -> Result<ResourceName, Status> {
    name.parent()
        .ok_or_else(|| Status::invalid_argument(format!("{name} has no parent collection")))
}
