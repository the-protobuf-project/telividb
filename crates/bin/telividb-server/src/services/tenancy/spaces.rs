//! The `Spaces` service.

use super::convert::{born, from_wire_protection, space};
use super::{TenancySvc, already_exists, not_found, now_millis, parse, require_id};
use crate::error::storage_status;
use telividb_buffers::protobuf::tenancy::v1::spaces_server::Spaces;
use telividb_buffers::protobuf::tenancy::v1::{
    CreateSpaceRequest, DeleteSpaceRequest, GetSpaceRequest, ListSpacesRequest, ListSpacesResponse,
    Space, UnlockSpaceRequest, UnlockSpaceResponse,
};
use telividb_core::{Protection, ResourceName, Space as DomainSpace};
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Spaces for TenancySvc {
    async fn create_space(
        &self,
        request: Request<CreateSpaceRequest>,
    ) -> Result<Response<Space>, Status> {
        let req = request.into_inner();
        require_id(&req.space_id, "space_id")?;
        let parent = parse(&req.parent)?;
        let name = parse(&format!("{}/spaces/{}", parent.as_str(), req.space_id))?;

        let organization = self
            .store
            .organization(&parent)
            .map_err(|e| storage_status(&e))?
            .ok_or_else(|| not_found(&parent))?;
        if !organization.lifecycle.is_live() {
            return Err(Status::failed_precondition(format!(
                "{parent} is deleted; undelete it before adding to it"
            )));
        }

        let payload = req.space.unwrap_or_default();
        let protection = from_wire_protection(payload.protection);

        // Refused rather than accepted-and-ignored. Nothing wraps a key yet, so
        // a space created as a vault would carry a protection state the engine
        // cannot honour — and rule 25 reserves the word for a cryptographic
        // guarantee. Saying so is the difference between a missing feature and
        // a false one.
        if matches!(protection, Protection::Vault | Protection::Sealed) {
            return Err(Status::unimplemented(
                "key wrapping is not implemented, so a vault or sealed space \
                 cannot be created yet — its contents would be no more \
                 protected than a private one. Use PROTECTION_PRIVATE for \
                 access control today.",
            ));
        }

        let mut projects = Vec::with_capacity(payload.projects.len());
        for project in &payload.projects {
            projects.push(parse(project)?);
        }

        let value = DomainSpace {
            name: name.clone(),
            display_name: payload.display_name,
            projects,
            protection,
            lifecycle: born(now_millis()),
        };

        match self
            .store
            .create_space(&value)
            .map_err(|e| storage_status(&e))?
        {
            true => Ok(Response::new(space(&value))),
            false => Err(already_exists(&name)),
        }
    }

    async fn get_space(
        &self,
        request: Request<GetSpaceRequest>,
    ) -> Result<Response<Space>, Status> {
        let name = parse(&request.into_inner().name)?;
        match self.store.space(&name).map_err(|e| storage_status(&e))? {
            Some(found) => Ok(Response::new(space(&found))),
            None => Err(not_found(&name)),
        }
    }

    async fn list_spaces(
        &self,
        request: Request<ListSpacesRequest>,
    ) -> Result<Response<ListSpacesResponse>, Status> {
        let parent = parse(&request.into_inner().parent)?;
        let found = self.store.spaces(false).map_err(|e| storage_status(&e))?;

        let prefix = format!("{}/spaces/", parent.as_str());
        Ok(Response::new(ListSpacesResponse {
            spaces: found
                .iter()
                .filter(|s| s.name.as_str().starts_with(&prefix))
                .map(space)
                .collect(),
            next_page_token: String::new(),
        }))
    }

    async fn unlock_space(
        &self,
        _request: Request<UnlockSpaceRequest>,
    ) -> Result<Response<UnlockSpaceResponse>, Status> {
        // Refusing is the honest answer while nothing wraps a key. A method
        // that reported success would tell a caller its contents had been
        // decrypted when they were never encrypted.
        Err(Status::unimplemented(
            "key wrapping is not implemented, so there is nothing to unlock",
        ))
    }

    async fn delete_space(
        &self,
        request: Request<DeleteSpaceRequest>,
    ) -> Result<Response<Space>, Status> {
        let name: ResourceName = parse(&request.into_inner().name)?;
        match self
            .store
            .delete_space(&name, now_millis())
            .map_err(|e| storage_status(&e))?
        {
            Some(deleted) => Ok(Response::new(space(&deleted))),
            None => Err(not_found(&name)),
        }
    }
}
