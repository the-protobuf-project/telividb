//! The `Projects` service.

use super::convert::{born, project};
use super::{TenancySvc, already_exists, not_found, now_millis, parse, require_id};
use crate::error::storage_status;
use telividb_buffers::protobuf::tenancy::v1::projects_server::Projects;
use telividb_buffers::protobuf::tenancy::v1::{
    CreateProjectRequest, DeleteProjectRequest, GetProjectRequest, ListProjectsRequest,
    ListProjectsResponse, Project,
};
use telividb_core::Project as DomainProject;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Projects for TenancySvc {
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<Project>, Status> {
        let req = request.into_inner();
        require_id(&req.project_id, "project_id")?;
        let parent = parse(&req.parent)?;
        let name = parse(&format!("{}/projects/{}", parent.as_str(), req.project_id))?;

        // The parent has to exist, and has to be live. Creating a project under
        // a tombstoned organization would make something that is unreachable
        // the moment it is written — and that would survive the organization's
        // own expiry.
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

        let payload = req.project.unwrap_or_default();
        let value = DomainProject {
            name: name.clone(),
            display_name: payload.display_name,
            lifecycle: born(now_millis()),
        };

        match self
            .store
            .create_project(&value)
            .map_err(|e| storage_status(&e))?
        {
            true => Ok(Response::new(project(&value))),
            false => Err(already_exists(&name)),
        }
    }

    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<Project>, Status> {
        let name = parse(&request.into_inner().name)?;
        match self.store.project(&name).map_err(|e| storage_status(&e))? {
            Some(found) => Ok(Response::new(project(&found))),
            None => Err(not_found(&name)),
        }
    }

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let req = request.into_inner();
        let parent = parse(&req.parent)?;
        let found = self
            .store
            .projects(req.show_deleted)
            .map_err(|e| storage_status(&e))?;

        // Filtered by parent here rather than in the store: the store keys by
        // resource name, and a name carries its parent, so a prefix is the
        // whole of the relationship.
        let prefix = format!("{}/projects/", parent.as_str());
        Ok(Response::new(ListProjectsResponse {
            projects: found
                .iter()
                .filter(|p| p.name.as_str().starts_with(&prefix))
                .map(project)
                .collect(),
            next_page_token: String::new(),
        }))
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<Project>, Status> {
        let name = parse(&request.into_inner().name)?;
        match self
            .store
            .delete_project(&name, now_millis())
            .map_err(|e| storage_status(&e))?
        {
            Some(deleted) => Ok(Response::new(project(&deleted))),
            None => Err(not_found(&name)),
        }
    }
}
