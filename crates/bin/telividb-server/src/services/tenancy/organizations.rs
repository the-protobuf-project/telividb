//! The `Organizations` service.

use super::convert::{born, organization};
use super::{TenancySvc, already_exists, not_found, now_millis, parse, require_id};
use crate::error::storage_status;
use telividb_buffers::protobuf::tenancy::v1::organizations_server::Organizations;
use telividb_buffers::protobuf::tenancy::v1::{
    CreateOrganizationRequest, DeleteOrganizationRequest, GetOrganizationRequest,
    ListOrganizationsRequest, ListOrganizationsResponse, Organization, UndeleteOrganizationRequest,
    UpdateOrganizationRequest,
};
use telividb_core::Organization as DomainOrganization;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Organizations for TenancySvc {
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<Organization>, Status> {
        let req = request.into_inner();
        require_id(&req.organization_id, "organization_id")?;
        let name = parse(&format!("organizations/{}", req.organization_id))?;

        let payload = req.organization.unwrap_or_default();
        let value = DomainOrganization {
            name: name.clone(),
            display_name: payload.display_name,
            lifecycle: born(now_millis()),
        };

        match self
            .store
            .create_organization(&value)
            .map_err(|e| storage_status(&e))?
        {
            true => Ok(Response::new(organization(&value))),
            false => Err(already_exists(&name)),
        }
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<Organization>, Status> {
        let name = parse(&request.into_inner().name)?;
        match self
            .store
            .organization(&name)
            .map_err(|e| storage_status(&e))?
        {
            Some(found) => Ok(Response::new(organization(&found))),
            None => Err(not_found(&name)),
        }
    }

    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        let req = request.into_inner();
        let found = self
            .store
            .organizations(req.show_deleted)
            .map_err(|e| storage_status(&e))?;

        Ok(Response::new(ListOrganizationsResponse {
            organizations: found.iter().map(organization).collect(),
            // Every organization fits in one page today. The token stays empty
            // rather than fabricated: a caller that follows one must reach the
            // end, and a token that never terminates is worse than none.
            next_page_token: String::new(),
        }))
    }

    async fn update_organization(
        &self,
        _request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<Organization>, Status> {
        Err(Status::unimplemented(
            "updating an organization is not implemented yet; \
             create, get, list, delete and undelete are",
        ))
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<Organization>, Status> {
        let name = parse(&request.into_inner().name)?;
        // Tombstoned, not removed. The tenant stops answering queries at once
        // and stays recoverable until it expires.
        match self
            .store
            .delete_organization(&name, now_millis())
            .map_err(|e| storage_status(&e))?
        {
            Some(deleted) => Ok(Response::new(organization(&deleted))),
            None => Err(not_found(&name)),
        }
    }

    async fn undelete_organization(
        &self,
        request: Request<UndeleteOrganizationRequest>,
    ) -> Result<Response<Organization>, Status> {
        let name = parse(&request.into_inner().name)?;
        match self
            .store
            .undelete_organization(&name, now_millis())
            .map_err(|e| storage_status(&e))?
        {
            Some(restored) => Ok(Response::new(organization(&restored))),
            None => Err(not_found(&name)),
        }
    }
}
