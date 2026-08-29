//! The tenancy tree over real gRPC.
//!
//! This is Phase 2's "done when": create an organization, a project and a
//! space, delete the organization, see it go, undelete it, see it return. It
//! exercises all four services and the soft-delete path in one pass, against a
//! `redb` file on disk rather than a stub.

mod support;

use support::server::TestServer;
use support::tenancy::make_org;
use telividb_buffers::protobuf::tenancy::v1::organizations_client::OrganizationsClient;
use telividb_buffers::protobuf::tenancy::v1::projects_client::ProjectsClient;
use telividb_buffers::protobuf::tenancy::v1::sessions_client::SessionsClient;
use telividb_buffers::protobuf::tenancy::v1::spaces_client::SpacesClient;
use telividb_buffers::protobuf::tenancy::v1::{
    CreateProjectRequest, CreateSessionRequest, CreateSpaceRequest, DeleteOrganizationRequest,
    GetOrganizationRequest, ListOrganizationsRequest, ListProjectsRequest, Project, Protection,
    Session, Space, UndeleteOrganizationRequest,
};

#[tokio::test]
async fn the_tree_is_created_deleted_and_restored() {
    let server = TestServer::start().await;
    let created = make_org(&server, "acme").await;
    assert_eq!(created.name, "organizations/acme");
    assert!(created.delete_time.is_none(), "a new tenant is not deleted");

    let mut projects = ProjectsClient::connect(server.url())
        .await
        .expect("connect");
    let project = projects
        .create_project(CreateProjectRequest {
            parent: "organizations/acme".to_owned(),
            project_id: "atlas".to_owned(),
            project: Some(Project {
                display_name: "Atlas".to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("create project")
        .into_inner();
    assert_eq!(project.name, "organizations/acme/projects/atlas");

    let mut spaces = SpacesClient::connect(server.url()).await.expect("connect");
    let space = spaces
        .create_space(CreateSpaceRequest {
            parent: "organizations/acme".to_owned(),
            space_id: "finance".to_owned(),
            space: Some(Space {
                display_name: "Finance".to_owned(),
                protection: Protection::Private as i32,
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("create space")
        .into_inner();
    assert_eq!(space.protection, Protection::Private as i32);
    assert!(!space.locked, "a private space is not key-wrapped");

    SessionsClient::connect(server.url())
        .await
        .expect("connect")
        .create_session(CreateSessionRequest {
            parent: "organizations/acme".to_owned(),
            session: Some(Session {
                display_name: "Tuesday".to_owned(),
                space: space.name.clone(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("create session");

    // Delete, and watch it leave the listing.
    let mut orgs = OrganizationsClient::connect(server.url())
        .await
        .expect("connect");
    let deleted = orgs
        .delete_organization(DeleteOrganizationRequest {
            name: "organizations/acme".to_owned(),
            ..Default::default()
        })
        .await
        .expect("delete")
        .into_inner();
    assert!(deleted.delete_time.is_some(), "a delete leaves a tombstone");
    assert!(
        deleted.expire_time.is_some(),
        "and an expiry to recover before"
    );

    let visible = orgs
        .list_organizations(ListOrganizationsRequest::default())
        .await
        .expect("list")
        .into_inner();
    assert!(
        visible.organizations.is_empty(),
        "a tombstone reached a list"
    );

    // Still fetchable by name, which is what makes undelete usable — a caller
    // has to be able to see what it is about to restore.
    let tombstoned = orgs
        .get_organization(GetOrganizationRequest {
            name: "organizations/acme".to_owned(),
        })
        .await
        .expect("get finds a tombstone")
        .into_inner();
    assert!(tombstoned.delete_time.is_some());

    let restored = orgs
        .undelete_organization(UndeleteOrganizationRequest {
            name: "organizations/acme".to_owned(),
        })
        .await
        .expect("undelete")
        .into_inner();
    assert!(
        restored.delete_time.is_none(),
        "a restore clears the tombstone"
    );

    // And the tree beneath it is still there — nothing was removed, so nothing
    // had to be rebuilt.
    let after = projects
        .list_projects(ListProjectsRequest {
            parent: "organizations/acme".to_owned(),
            ..Default::default()
        })
        .await
        .expect("list projects")
        .into_inner();
    assert_eq!(after.projects.len(), 1);
}
