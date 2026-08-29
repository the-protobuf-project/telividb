//! The `Models` trait implementation.
//!
//! Each method forwards to the logic beside it. Kept separate so the trait's
//! shape — which `tonic` dictates — does not crowd out what the methods do.

use super::ModelsSvc;
use telividb_buffers::protobuf::models::v1::models_server::Models;
use telividb_buffers::protobuf::models::v1::{
    CatalogModel, CreateModelInstallationRequest, DeleteModelInstallationRequest,
    GetCatalogModelRequest, GetModelInstallationRequest, ListCatalogModelsRequest,
    ListCatalogModelsResponse, ListModelInstallationsRequest, ListModelInstallationsResponse,
    ModelInstallation,
};
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Models for ModelsSvc {
    async fn list_catalog_models(
        &self,
        request: Request<ListCatalogModelsRequest>,
    ) -> Result<Response<ListCatalogModelsResponse>, Status> {
        self.list_catalog(request)
    }

    async fn get_catalog_model(
        &self,
        request: Request<GetCatalogModelRequest>,
    ) -> Result<Response<CatalogModel>, Status> {
        self.get_catalog(request)
    }

    async fn create_model_installation(
        &self,
        request: Request<CreateModelInstallationRequest>,
    ) -> Result<Response<ModelInstallation>, Status> {
        self.create_install(request)
    }

    async fn get_model_installation(
        &self,
        request: Request<GetModelInstallationRequest>,
    ) -> Result<Response<ModelInstallation>, Status> {
        self.get_install(request)
    }

    async fn list_model_installations(
        &self,
        request: Request<ListModelInstallationsRequest>,
    ) -> Result<Response<ListModelInstallationsResponse>, Status> {
        self.list_installs(request)
    }

    async fn delete_model_installation(
        &self,
        request: Request<DeleteModelInstallationRequest>,
    ) -> Result<Response<ModelInstallation>, Status> {
        self.delete_install(request)
    }
}
