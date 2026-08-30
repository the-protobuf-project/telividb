//! Organizations, projects and spaces.
//!
//! Every method here is a thin call: build the request, send it, convert the
//! answer. Listing pages transparently — a caller asking for "the projects"
//! wants all of them, and a page token is an artefact of the transport rather
//! than something a window should carry.

use crate::error::Result;
use crate::tenancy_types::{Organization, Project, Protection, Space};
use telividb_buffers::protobuf::tenancy::v1 as wire;

/// How many to ask for per page. Large enough that one round trip usually
/// suffices, small enough that a huge tenant does not arrive in one message.
const PAGE_SIZE: i32 = 200;

impl crate::Client {
    /// Every organization, following pagination to the end.
    pub async fn list_organizations(&self) -> Result<Vec<Organization>> {
        let mut out = Vec::new();
        let mut page_token = String::new();
        loop {
            let response = self
                .organizations
                .clone()
                .list_organizations(wire::ListOrganizationsRequest {
                    page_size: PAGE_SIZE,
                    page_token: page_token.clone(),
                    filter: String::new(),
                    show_deleted: true,
                })
                .await?
                .into_inner();
            out.extend(response.organizations.into_iter().map(Organization::from));
            if response.next_page_token.is_empty() {
                return Ok(out);
            }
            page_token = response.next_page_token;
        }
    }

    /// Create an organization.
    ///
    /// `id` becomes the last segment of the resource name and cannot be changed
    /// afterwards; `display_name` is what a person reads and can.
    pub async fn create_organization(
        &self,
        id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Result<Organization> {
        let organization = self
            .organizations
            .clone()
            .create_organization(wire::CreateOrganizationRequest {
                organization_id: id.into(),
                organization: Some(wire::Organization {
                    display_name: display_name.into(),
                    ..Default::default()
                }),
                request_id: String::new(),
            })
            .await?
            .into_inner();
        Ok(organization.into())
    }

    /// Soft-delete an organization. It stays until its expiry passes.
    pub async fn delete_organization(&self, name: impl Into<String>) -> Result<Organization> {
        let organization = self
            .organizations
            .clone()
            .delete_organization(wire::DeleteOrganizationRequest {
                name: name.into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        Ok(organization.into())
    }

    /// Restore a soft-deleted organization.
    pub async fn undelete_organization(&self, name: impl Into<String>) -> Result<Organization> {
        let organization = self
            .organizations
            .clone()
            .undelete_organization(wire::UndeleteOrganizationRequest { name: name.into() })
            .await?
            .into_inner();
        Ok(organization.into())
    }

    /// Every project under one organization.
    pub async fn list_projects(&self, parent: impl Into<String>) -> Result<Vec<Project>> {
        let parent = parent.into();
        let mut out = Vec::new();
        let mut page_token = String::new();
        loop {
            let response = self
                .projects
                .clone()
                .list_projects(wire::ListProjectsRequest {
                    parent: parent.clone(),
                    page_size: PAGE_SIZE,
                    page_token: page_token.clone(),
                    ..Default::default()
                })
                .await?
                .into_inner();
            out.extend(response.projects.into_iter().map(Project::from));
            if response.next_page_token.is_empty() {
                return Ok(out);
            }
            page_token = response.next_page_token;
        }
    }

    /// Create a project under one organization.
    pub async fn create_project(
        &self,
        parent: impl Into<String>,
        id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Result<Project> {
        let project = self
            .projects
            .clone()
            .create_project(wire::CreateProjectRequest {
                parent: parent.into(),
                project_id: id.into(),
                project: Some(wire::Project {
                    display_name: display_name.into(),
                    ..Default::default()
                }),
                request_id: String::new(),
            })
            .await?
            .into_inner();
        Ok(project.into())
    }

    /// Every space under one organization.
    pub async fn list_spaces(&self, parent: impl Into<String>) -> Result<Vec<Space>> {
        let parent = parent.into();
        let mut out = Vec::new();
        let mut page_token = String::new();
        loop {
            let response = self
                .spaces
                .clone()
                .list_spaces(wire::ListSpacesRequest {
                    parent: parent.clone(),
                    page_size: PAGE_SIZE,
                    page_token: page_token.clone(),
                    ..Default::default()
                })
                .await?
                .into_inner();
            out.extend(response.spaces.into_iter().map(Space::from));
            if response.next_page_token.is_empty() {
                return Ok(out);
            }
            page_token = response.next_page_token;
        }
    }

    /// Create a space, declaring its protection.
    ///
    /// Protection is required at creation and never changes: it decides which
    /// segments the space's contents are routed to, so altering it later would
    /// mean rewriting all of them.
    pub async fn create_space(
        &self,
        parent: impl Into<String>,
        id: impl Into<String>,
        display_name: impl Into<String>,
        protection: Protection,
    ) -> Result<Space> {
        let space = self
            .spaces
            .clone()
            .create_space(wire::CreateSpaceRequest {
                parent: parent.into(),
                space_id: id.into(),
                space: Some(wire::Space {
                    display_name: display_name.into(),
                    protection: protection.as_wire(),
                    ..Default::default()
                }),
                request_id: String::new(),
            })
            .await?
            .into_inner();
        Ok(space.into())
    }
}
