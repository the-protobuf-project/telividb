//! The commands themselves.
//!
//! In a module of their own because `#[tauri::command]` generates a helper
//! macro beside each function, and at a crate root those collide with the
//! re-exports. Naming them through `$crate::commands::` in the handler list
//! keeps both reachable.
//!
//! Every function here forwards. None of them decide.

use crate::dto::{CollectionSummary, SearchRequest, SearchResponse};
use crate::state::AppState;

/// Collections this engine holds.
#[tauri::command]
pub async fn list_collections(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<CollectionSummary>, String> {
    let mut client = state.client();
    let ids = client.list_collections().await.map_err(say)?;
    Ok(ids.into_iter().map(CollectionSummary::new).collect())
}

/// Search one collection.
///
/// Text and vector queries are both the client's business; this picks between
/// them on which field the request filled, and does nothing else.
#[tauri::command]
pub async fn search(
    state: tauri::State<'_, AppState>,
    request: SearchRequest,
) -> Result<SearchResponse, String> {
    let mut collection = state.client().collection(&request.collection);
    let results = match &request.text {
        Some(text) => collection
            .search_text(&request.field, text, request.k)
            .await
            .map_err(say)?,
        None => collection
            .search(&request.field, &request.vector, request.k)
            .await
            .map_err(say)?,
    };
    Ok(SearchResponse::from(results))
}

/// Where the engine is listening, so an external tool can reach the same one.
#[tauri::command]
pub fn engine_address(state: tauri::State<'_, AppState>) -> String {
    state.addr().to_string()
}

/// Render a client error for the window.
///
/// Tauri carries a command failure as a string, so the alternative to this is
/// `unwrap` — and a window that vanishes on a refused query is worse than one
/// that says why.
fn say(error: telividb_client::Error) -> String {
    error.to_string()
}
