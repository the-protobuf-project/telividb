//! Fixtures for the tenancy services.

#![allow(dead_code)]

use telividb_buffers::protobuf::tenancy::v1::organizations_client::OrganizationsClient;
use telividb_buffers::protobuf::tenancy::v1::{CreateOrganizationRequest, Organization};

use super::server::TestServer;

/// Create `organizations/{id}` and return it.
///
/// Every tenancy test needs one, because an organization is the root of the
/// tree and nothing else can be addressed without it.
pub async fn make_org(server: &TestServer, id: &str) -> Organization {
    OrganizationsClient::connect(server.url())
        .await
        .expect("connect")
        .create_organization(CreateOrganizationRequest {
            organization_id: id.to_owned(),
            organization: Some(Organization {
                display_name: id.to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("create organization")
        .into_inner()
}
