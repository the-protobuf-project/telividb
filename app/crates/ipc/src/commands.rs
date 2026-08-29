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
    // Exactly one of the two carries the query, and the window must not pick
    // for the caller. Sending both would silently drop the vector; sending
    // neither reaches the server as an empty vector, which it refuses in terms
    // that describe the wire rather than the mistake.
    //
    // This refuses rather than decides, which is the line this layer holds: a
    // shim may reject an ambiguous request, but it may not resolve one.
    let has_text = request.text.as_ref().is_some_and(|t| !t.is_empty());
    let has_vector = !request.vector.is_empty();
    match (has_text, has_vector) {
        (true, true) => {
            return Err("give either `text` or `vector`, not both: \
                        each names a different query, and the server encodes \
                        text through the field's own model."
                .to_owned());
        }
        (false, false) => {
            return Err("a search needs either `text` to encode or a `vector` \
                        to match against."
                .to_owned());
        }
        _ => {}
    }

    let mut collection = state.client().collection(&request.collection);
    // Dispatched on the same reading of the request the check above used. On
    // `request.text` directly, a present-but-empty string would pass validation
    // as a vector search and then be sent as a text query of "".
    let results = match request.text.as_deref().filter(|_| has_text) {
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
