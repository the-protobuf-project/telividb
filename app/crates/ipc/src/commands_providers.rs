//! Provider settings: which hosts exist, and the keys that reach them.

use crate::dto_providers::ProviderDto;
use telividb_providers::{KeychainStore, PROVIDERS, SecretStore, provider};

/// What this app's keychain entries are filed under.
///
/// A person reads this in Keychain Access, so it is the product's name rather
/// than a crate's.
const SERVICE: &str = "telividb";

/// The store these commands read and write.
fn store() -> KeychainStore {
    KeychainStore::new(SERVICE)
}

/// Every provider, with whether it is ready to use.
#[tauri::command]
pub async fn list_providers() -> Vec<ProviderDto> {
    let store = store();
    PROVIDERS
        .iter()
        .map(|p| ProviderDto::new(p, store.has(p.id)))
        .collect()
}

/// Store a credential for one provider, replacing any already there.
#[tauri::command]
pub async fn store_provider_key(id: String, credential: String) -> Result<(), String> {
    provider(&id).ok_or_else(|| format!("no provider called {id:?}"))?;
    store()
        .set(&id, credential.trim())
        .map_err(|e| e.to_string())
}

/// Forget a provider's credential. Forgetting an absent one is not an error.
#[tauri::command]
pub async fn forget_provider_key(id: String) -> Result<(), String> {
    provider(&id).ok_or_else(|| format!("no provider called {id:?}"))?;
    store().clear(&id).map_err(|e| e.to_string())
}

/// Hand the window a credential so it can call the provider.
///
/// **This is the command that puts a key in the webview**, and it is the whole
/// cost of answering in the window rather than in the engine. It is a named
/// command rather than a general secret reader so the exposure has one call site
/// that can be found, audited, and — if answering ever moves behind a proxy —
/// deleted in one place.
///
/// A local provider has no secret to leak: what comes back is its address, and an
/// absent entry means the default one.
#[tauri::command]
pub async fn provider_credential(id: String) -> Result<String, String> {
    let p = provider(&id).ok_or_else(|| format!("no provider called {id:?}"))?;
    let stored = store().get(&id).map_err(|e| e.to_string())?;
    match stored {
        Some(secret) => Ok(secret),
        None if !p.needs_key() => Ok(String::new()),
        None => Err(format!(
            "{} needs an API key before it can answer; add one in settings",
            p.display_name
        )),
    }
}
