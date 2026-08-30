//! What the window is told about a provider.

use serde::Serialize;

/// A model host, as the settings panel renders it.
///
/// Mirrors `telividb_providers::Provider` with one field added and none removed:
/// [`configured`](Self::configured) answers "can this be chosen yet?" without the
/// window having to ask for a credential to find out.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDto {
    /// Stable id, used to pick an adapter and to key the keychain entry.
    pub id: String,
    /// Name shown to a person.
    pub display_name: String,
    /// `"local"` or `"remote"`, deciding what may be sent.
    pub locality: String,
    /// What it is, in the terms that decide whether to use it.
    pub note: String,
    /// Models it offers, most useful first.
    pub models: Vec<String>,
    /// What its credential looks like, shown as placeholder text.
    pub credential_hint: String,
    /// Whether it can be used now. Never the credential itself.
    pub configured: bool,
}

impl ProviderDto {
    /// Describe one provider, reading only whether a key is present.
    ///
    /// A local provider is reached by address rather than by key, so it counts as
    /// configured with nothing stored — otherwise Ollama would appear unusable
    /// until someone invented a secret for it.
    pub fn new(provider: &telividb_providers::Provider, has_key: bool) -> Self {
        Self {
            id: provider.id.to_owned(),
            display_name: provider.display_name.to_owned(),
            locality: match provider.locality {
                telividb_providers::Locality::Local => "local",
                telividb_providers::Locality::Remote => "remote",
            }
            .to_owned(),
            note: provider.note.to_owned(),
            models: provider.models.iter().map(|m| (*m).to_string()).collect(),
            credential_hint: provider.credential_hint.to_owned(),
            configured: !provider.needs_key() || has_key,
        }
    }
}
