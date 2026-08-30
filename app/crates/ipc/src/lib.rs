//! The window's half of the bridge.
//!
//! # Every function here forwards, and none of them decide
//!
//! A command translates a JSON payload into a client call and the answer back
//! again. That is the whole contract. The engine's rules — which field a query
//! runs against, what a score means, whether a result set is complete — live in
//! the engine, and a second copy here would be a second answer to the same
//! question that nothing keeps in step.
//!
//! It is enforced by shape rather than by care: these functions hold a
//! [`telividb_client::Client`] and nothing else, so there is no state to make a
//! decision from. Anything richer would need a dependency this crate does not
//! have, and adding one is a visible change to a manifest rather than a quiet
//! change to a function.
//!
//! # Types, and why they are written out
//!
//! The payloads below mirror the client's small surface rather than the wire
//! protos. A search needs a collection, a field, a query and a `k`; spelling
//! that out is a dozen lines, while generating the whole proto surface to reach
//! it would be hundreds of types the window never mentions. When the app grows
//! past what the client exposes, `protoc-gen-es` generates these instead — and
//! that is the moment to switch, not before.

pub mod commands;
pub mod commands_models;
pub mod commands_providers;
pub mod commands_tenancy;
mod dto;
pub mod dto_models;
pub mod dto_providers;
pub mod dto_system;
pub mod dto_tenancy;
pub mod presets;
mod state;

pub use dto::{
    Capabilities, CollectionSummary, CreateCollectionRequest, ImportRequest, ImportResponse,
    ImportRow, PointRow, SearchHit, SearchRequest, SearchResponse,
};
pub use presets::{PRESETS, Preset};
pub use state::AppState;

/// Every command the window may invoke.
///
/// One list, so a command cannot be added to the backend and forgotten in the
/// handler — or registered without being reachable.
#[macro_export]
macro_rules! commands {
    () => {
        tauri::generate_handler![
            $crate::commands::list_collections,
            $crate::commands::search,
            $crate::commands::engine_address,
            $crate::commands::capabilities,
            $crate::commands::create_collection,
            $crate::commands::list_presets,
            $crate::commands::import_points,
            $crate::commands::list_points,
            $crate::commands_models::list_models,
            $crate::commands_models::install_model,
            $crate::commands_models::installation,
            $crate::commands_models::cancel_installation,
            $crate::commands_providers::list_providers,
            $crate::commands_providers::store_provider_key,
            $crate::commands_providers::forget_provider_key,
            $crate::commands_providers::provider_credential,
            $crate::commands_tenancy::list_organizations,
            $crate::commands_tenancy::create_organization,
            $crate::commands_tenancy::delete_organization,
            $crate::commands_tenancy::undelete_organization,
            $crate::commands_tenancy::list_projects,
            $crate::commands_tenancy::create_project,
            $crate::commands_tenancy::list_spaces,
            $crate::commands_tenancy::create_space,
        ]
    };
}
