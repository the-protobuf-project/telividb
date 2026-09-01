//! The model exchange: what the window renders, and how the wire becomes it.
//!
//! Its own file rather than part of `dto.rs` because these types travel with
//! their conversions — the interesting decision is here, not in the shape: the
//! wire carries an installation state as an integer, and a template comparing
//! against `2` is a template nobody can read.

use telividb_client::wire::models::v1 as wire;

/// One model the catalog offers.
///
/// A projection of the wire message, not a re-derivation: every field is copied
/// across so the window and `grpcurl` describe the same model. Sizes arrive as
/// `u64` rather than the wire's `i64` because a negative one is not a state the
/// window can render, and the conversion is the right place to say so.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogModel {
    /// Catalog id, which is what an install call names.
    pub id: String,
    /// Name shown in the list.
    pub display_name: String,
    /// What the model is good for, in a sentence.
    pub description: String,
    /// Page for the weights, for checking licence and provenance at the source.
    pub repository_uri: String,
    /// Exact size, for a progress bar that ends where it should.
    pub size_bytes: u64,
    /// Components per vector.
    pub dimensions: u32,
    /// Longest input in tokens.
    pub context_length: u32,
    /// SPDX identifier for the weights' licence.
    pub license: String,
    /// Whether this is the default offer.
    pub recommended: bool,
    /// `text`, `image`, `audio` or `video`.
    ///
    /// Carried so the window can say which modalities have an encoder behind
    /// them and which do not. Every catalog entry is `text` today; the other
    /// three exist in the schema and have no model, which is a fact the window
    /// should state rather than imply by omission.
    pub modality: String,
    /// Whether the file is on disk at its full length.
    pub installed: bool,
    /// Whether its weights are loaded and able to embed right now.
    ///
    /// Separate from [`installed`](Self::installed) because the gap between the
    /// two is seconds on a large model, and it is the window in which text is
    /// refused.
    pub resident: bool,
}

/// How far an installation has got.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installation {
    /// Resource name, which is what a poll names.
    pub name: String,
    /// One of `pending`, `downloading`, `verifying`, `succeeded`, `failed`,
    /// `cancelled`.
    ///
    /// A string rather than the wire's integer, so a template can compare
    /// against something readable and an unknown value is visible rather than
    /// rendering as a number.
    pub state: String,
    /// Bytes written so far, including any resumed from an earlier attempt.
    pub progress_bytes: u64,
    /// Total expected, from the catalog entry.
    ///
    /// The denominator of the progress bar. Zero would make it divide by
    /// nothing, so the window checks before using it.
    pub total_bytes: u64,
    /// Why it stopped, when it failed.
    pub error: String,
}

impl CatalogModel {
    /// Project a wire catalog model.
    pub(crate) fn from_wire(source: &wire::CatalogModel) -> Self {
        Self {
            // The window installs by id, so the `catalogModels/` prefix is
            // stripped here rather than in every call site that uses it.
            id: source
                .name
                .strip_prefix("catalogModels/")
                .unwrap_or(&source.name)
                .to_owned(),
            display_name: source.display_name.clone(),
            description: source.description.clone(),
            repository_uri: source.repository_uri.clone(),
            size_bytes: source.size_bytes.max(0) as u64,
            dimensions: source.dimensions.max(0) as u32,
            context_length: source.context_length.max(0) as u32,
            license: source.license.clone(),
            recommended: source.recommended,
            modality: modality_name(source.modality),
            installed: source.installed,
            resident: source.resident,
        }
    }
}

impl Installation {
    /// Project a wire installation.
    pub(crate) fn from_wire(source: &wire::ModelInstallation) -> Self {
        Self {
            name: source.name.clone(),
            state: state_name(source.state),
            progress_bytes: source.progress_bytes.max(0) as u64,
            total_bytes: source.total_bytes.max(0) as u64,
            error: source.error.clone(),
        }
    }
}

/// The state as a name a template can compare against.
///
/// An unrecognised value becomes `"unknown"` rather than a number: the window
/// then shows something obviously wrong instead of a state it silently treats
/// as "not finished", which would leave a progress bar spinning forever.
fn state_name(value: i32) -> String {
    match wire::InstallationState::try_from(value) {
        Ok(wire::InstallationState::Pending) => "pending",
        Ok(wire::InstallationState::Downloading) => "downloading",
        Ok(wire::InstallationState::Verifying) => "verifying",
        Ok(wire::InstallationState::Succeeded) => "succeeded",
        Ok(wire::InstallationState::Failed) => "failed",
        Ok(wire::InstallationState::Cancelled) => "cancelled",
        Ok(wire::InstallationState::Unspecified) | Err(_) => "unknown",
    }
    .to_owned()
}

/// The wire modality as the window names it.
///
/// An unrecognised value reads as `text` rather than as an error: the field is
/// descriptive, and a modality this build does not know is still more likely to
/// be text than to be nothing.
fn modality_name(value: i32) -> String {
    // The tags from `catalog_model.proto`, matched by number rather than by
    // importing the generated enum: this crate depends on the SDK, not on the
    // buffers crate, and a bridge should not reach past its own client for a
    // four-armed match. Rule 40 keeps these numbers permanent.
    match value {
        2 => "image",
        3 => "audio",
        4 => "video",
        _ => "text",
    }
    .to_owned()
}
