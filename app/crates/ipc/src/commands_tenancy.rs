//! Organizations, projects and spaces. Every function here forwards.

use crate::dto_tenancy::{OrganizationDto, ProjectDto, SpaceDto};
use crate::state::AppState;
use telividb_client::Protection;

/// Turn a client error into the sentence the window shows.
fn say(error: telividb_client::Error) -> String {
    error.to_string()
}

/// Read a protection from the window's string form.
///
/// Rejects an unknown value rather than defaulting. Protection is fixed at
/// creation and decides segment routing, so guessing here would silently create
/// a space with weaker protection than the caller asked for.
fn protection(value: &str) -> Result<Protection, String> {
    match value {
        "none" => Ok(Protection::None),
        "private" => Ok(Protection::Private),
        "vault" => Ok(Protection::Vault),
        "sealed" => Ok(Protection::Sealed),
        other => Err(format!(
            "unknown protection {other:?}; expected none, private, vault or sealed"
        )),
    }
}

/// Every organization this engine holds, soft-deleted ones included.
#[tauri::command]
pub async fn list_organizations(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<OrganizationDto>, String> {
    let client = state.client();
    let found = client.list_organizations().await.map_err(say)?;
    Ok(found.into_iter().map(OrganizationDto::from).collect())
}

/// Create an organization.
#[tauri::command]
pub async fn create_organization(
    state: tauri::State<'_, AppState>,
    id: String,
    display_name: String,
) -> Result<OrganizationDto, String> {
    let client = state.client();
    let created = client
        .create_organization(id, display_name)
        .await
        .map_err(say)?;
    Ok(created.into())
}

/// Soft-delete an organization. It remains until its expiry passes.
#[tauri::command]
pub async fn delete_organization(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<OrganizationDto, String> {
    let client = state.client();
    let deleted = client.delete_organization(name).await.map_err(say)?;
    Ok(deleted.into())
}

/// Restore a soft-deleted organization.
#[tauri::command]
pub async fn undelete_organization(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<OrganizationDto, String> {
    let client = state.client();
    let restored = client.undelete_organization(name).await.map_err(say)?;
    Ok(restored.into())
}

/// Every project under one organization.
#[tauri::command]
pub async fn list_projects(
    state: tauri::State<'_, AppState>,
    parent: String,
) -> Result<Vec<ProjectDto>, String> {
    let client = state.client();
    let found = client.list_projects(parent).await.map_err(say)?;
    Ok(found.into_iter().map(ProjectDto::from).collect())
}

/// Create a project under one organization.
#[tauri::command]
pub async fn create_project(
    state: tauri::State<'_, AppState>,
    parent: String,
    id: String,
    display_name: String,
) -> Result<ProjectDto, String> {
    let client = state.client();
    let created = client
        .create_project(parent, id, display_name)
        .await
        .map_err(say)?;
    Ok(created.into())
}

/// Every space under one organization.
#[tauri::command]
pub async fn list_spaces(
    state: tauri::State<'_, AppState>,
    parent: String,
) -> Result<Vec<SpaceDto>, String> {
    let client = state.client();
    let found = client.list_spaces(parent).await.map_err(say)?;
    Ok(found.into_iter().map(SpaceDto::from).collect())
}

/// Create a space, declaring its protection.
#[tauri::command]
pub async fn create_space(
    state: tauri::State<'_, AppState>,
    parent: String,
    id: String,
    display_name: String,
    protection_kind: String,
) -> Result<SpaceDto, String> {
    let client = state.client();
    let created = client
        .create_space(parent, id, display_name, protection(&protection_kind)?)
        .await
        .map_err(say)?;
    Ok(created.into())
}
