//! What tenancy refuses, and why each refusal is better than accepting.
//!
//! Separate from the round trip in `tenancy.rs` because it is a different
//! question: that file asks whether the happy path works, this one asks whether
//! the two states that cannot be honoured are reported rather than absorbed
//! (CLAUDE.md rule 49).

mod support;

use support::server::TestServer;
use support::tenancy::make_org;
use telividb_buffers::protobuf::tenancy::v1::organizations_client::OrganizationsClient;
use telividb_buffers::protobuf::tenancy::v1::projects_client::ProjectsClient;
use telividb_buffers::protobuf::tenancy::v1::spaces_client::SpacesClient;
use telividb_buffers::protobuf::tenancy::v1::{
    CreateProjectRequest, CreateSpaceRequest, DeleteOrganizationRequest, Project, Protection, Space,
};

#[tokio::test]
async fn a_vault_is_refused_rather_than_promised() {
    let server = TestServer::start().await;
    make_org(&server, "acme").await;

    // Nothing wraps a key yet. Accepting this would create a space whose
    // protection the engine cannot honour — "vault" names a cryptographic
    // guarantee (rule 25), and a space claiming one without having it is worse
    // than no space at all.
    let status = SpacesClient::connect(server.url())
        .await
        .expect("connect")
        .create_space(CreateSpaceRequest {
            parent: "organizations/acme".to_owned(),
            space_id: "board".to_owned(),
            space: Some(Space {
                protection: Protection::Vault as i32,
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect_err("a vault cannot be created yet");
    assert_eq!(status.code(), tonic::Code::Unimplemented);
    assert!(status.message().contains("key wrapping"));
}

#[tokio::test]
async fn a_project_under_a_deleted_organization_is_refused() {
    let server = TestServer::start().await;
    make_org(&server, "acme").await;

    OrganizationsClient::connect(server.url())
        .await
        .expect("connect")
        .delete_organization(DeleteOrganizationRequest {
            name: "organizations/acme".to_owned(),
            ..Default::default()
        })
        .await
        .expect("delete");

    // Otherwise the project is unreachable the moment it is written, and would
    // outlive the expiry of the organization it hangs from.
    let status = ProjectsClient::connect(server.url())
        .await
        .expect("connect")
        .create_project(CreateProjectRequest {
            parent: "organizations/acme".to_owned(),
            project_id: "atlas".to_owned(),
            project: Some(Project::default()),
            ..Default::default()
        })
        .await
        .expect_err("a deleted tenant takes no new children");
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
}
