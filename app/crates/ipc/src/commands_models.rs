//! Commands for the model catalog.
//!
//! Split from `commands.rs` for the reason that file gives for existing at all:
//! `#[tauri::command]` generates a macro beside each function, so the functions
//! are grouped where a reader looks for them rather than in one list. Every
//! function here forwards. None of them decide.

use crate::dto_models::{CatalogModel, Installation};
use crate::state::AppState;

/// Turn an engine failure into the sentence the window shows.
fn say(error: telividb_client::Error) -> String {
    error.to_string()
}

/// Models the engine offers to install.
#[tauri::command]
pub async fn list_models(state: tauri::State<'_, AppState>) -> Result<Vec<CatalogModel>, String> {
    let mut client = state.client();
    let models = client.list_models().await.map_err(say)?;
    Ok(models.iter().map(CatalogModel::from_wire).collect())
}

/// Begin installing a model.
///
/// Returns the handle immediately; the window polls it. Calling this twice for
/// one model returns the running installation rather than starting a second
/// transfer, so the button needs no guard of its own.
#[tauri::command]
pub async fn install_model(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Installation, String> {
    let mut client = state.client();
    let started = client.install_model(&id).await.map_err(say)?;
    Ok(Installation::from_wire(&started))
}

/// How far an installation has got.
#[tauri::command]
pub async fn installation(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<Installation, String> {
    let mut client = state.client();
    let current = client.installation(&name).await.map_err(say)?;
    Ok(Installation::from_wire(&current))
}

/// Stop an installation, keeping its partial file so a retry resumes.
#[tauri::command]
pub async fn cancel_installation(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<Installation, String> {
    let mut client = state.client();
    let stopped = client.cancel_installation(&name).await.map_err(say)?;
    Ok(Installation::from_wire(&stopped))
}
