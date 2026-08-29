//! Catalog types to wire types.

use super::catalog_name;
use telividb_buffers::protobuf::models::v1 as wire;
use telividb_core::Modality;
use telividb_models::{CatalogEntry, ModelStore};

/// One catalog entry, as the API describes it.
///
/// `installed` is computed rather than stored, and it verifies the digest
/// rather than checking that a path exists: the interesting failure is a file
/// that is present and wrong — truncated by a full disk, or replaced — and
/// reporting that as installed would offer a model that fails at load.
pub(super) fn entry(source: &CatalogEntry, store: &ModelStore) -> wire::CatalogModel {
    wire::CatalogModel {
        name: catalog_name(&source.id),
        display_name: source.display_name.clone(),
        description: source.description.clone(),
        repository_uri: source.repository_url(),
        digest: source.digest.to_hex(),
        size_bytes: source.size_bytes as i64,
        modality: modality(source.modality) as i32,
        architecture: source.architecture.as_str().to_owned(),
        dimensions: source.dimensions as i32,
        context_length: source.context_length as i32,
        quantization: source.quantization.clone(),
        license: source.license.clone(),
        recommended: source.recommended,
        installed: store.is_installed(source),
    }
}

/// The wire enum for a modality.
fn modality(source: Modality) -> wire::Modality {
    match source {
        Modality::Text => wire::Modality::Text,
        Modality::Image => wire::Modality::Image,
        Modality::Audio => wire::Modality::Audio,
        Modality::Video => wire::Modality::Video,
    }
}
