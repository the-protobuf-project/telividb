//! Catalog types to wire types.

use super::catalog_name;
use telividb_buffers::protobuf::models::v1 as wire;
use telividb_core::Modality;
use telividb_models::{CatalogEntry, ModelStore};

/// One catalog entry, as the API describes it.
///
/// `installed` and `resident` are both computed rather than stored, and they
/// answer different questions: whether the file is on disk, and whether its
/// weights are loaded and able to serve a request. The gap between them is
/// seconds on a large model, and a caller that conflates them sends text into
/// that window and is refused.
pub(super) fn entry(
    source: &CatalogEntry,
    store: &ModelStore,
    resident: Option<&str>,
) -> wire::CatalogModel {
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
        // Named rather than counted: several models can be installed and only
        // one is loaded, and a caller deciding whether it can send text needs
        // to know which.
        resident: resident == Some(source.id.as_str()),
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
