//! Organizations, projects and spaces, as the window renders them.
//!
//! Resource names cross this boundary whole (`organizations/acme/projects/web`)
//! rather than being split into parts. They are the only portable identity the
//! server accepts, so a window that reassembled one from pieces would be
//! reimplementing a format the server already owns.

use serde::Serialize;

/// A tenant: the top of the hierarchy, and what every other resource hangs off.
///
/// The counts are the server's, not a length the window computed — a project the
/// caller may not see still counts, so recomputing them here would quietly
/// disagree with the engine.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationDto {
    /// Resource name, `organizations/{organization}`.
    pub name: String,
    /// What a person calls it.
    pub display_name: String,
    /// How many projects it holds.
    pub project_count: i32,
    /// How many spaces it holds.
    pub space_count: i32,
    /// Whether it is soft-deleted and awaiting purge.
    pub deleted: bool,
}

impl From<telividb_client::Organization> for OrganizationDto {
    fn from(o: telividb_client::Organization) -> Self {
        Self {
            name: o.name,
            display_name: o.display_name,
            project_count: o.project_count,
            space_count: o.space_count,
            deleted: o.deleted,
        }
    }
}

/// A unit of work inside an organization.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    /// Resource name, `organizations/{organization}/projects/{project}`.
    pub name: String,
    /// What a person calls it.
    pub display_name: String,
    /// Whether it is soft-deleted and awaiting purge.
    pub deleted: bool,
}

impl From<telividb_client::Project> for ProjectDto {
    fn from(p: telividb_client::Project) -> Self {
        Self {
            name: p.name,
            display_name: p.display_name,
            deleted: p.deleted,
        }
    }
}

/// A protection boundary, which may span several projects.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceDto {
    /// Resource name, `organizations/{organization}/spaces/{space}`.
    pub name: String,
    /// What a person calls it.
    pub display_name: String,
    /// Projects this space serves, by resource name.
    pub projects: Vec<String>,
    /// `none`, `private`, `vault` or `sealed` — the wire's own words.
    pub protection: String,
    /// Whether its key is currently unavailable.
    pub locked: bool,
    /// Whether it is soft-deleted and awaiting purge.
    pub deleted: bool,
}

impl From<telividb_client::Space> for SpaceDto {
    fn from(s: telividb_client::Space) -> Self {
        Self {
            name: s.name,
            display_name: s.display_name,
            projects: s.projects,
            protection: s.protection.as_str().to_owned(),
            locked: s.locked,
            deleted: s.deleted,
        }
    }
}
