//! Collection schemas the app ships with.
//!
//! # Why the app carries compiled bytes
//!
//! `CreateCollection` takes a compiled `FileDescriptorSet`, not a `.proto`:
//! the engine never parses schema text, it consumes descriptor bytes and their
//! digest becomes the schema's identity. A window cannot compile a `.proto`
//! — that needs `buf` or `protoc` on the machine — so a desktop app that could
//! only create collections from user-supplied schemas could not create one at
//! all on a fresh install.
//!
//! These are the answer: two real schemas, compiled by `buf` and committed
//! beside their source in `app/presets/`. Regenerate with
//! `cargo xtask gen-presets`.

use serde::Serialize;
use telividb_client::{Metric, NewCollection};

/// Dimensions every preset's text field declares.
///
/// 768 is what the encoder-style models this engine loads produce — bge, e5,
/// gte and nomic all land there. A field is bound to one model identity, so
/// this is not a default so much as a commitment, and a preset that guessed
/// wrong would produce a collection nothing could write to.
const TEXT_DIMENSIONS: usize = 768;

/// A schema the app can create a collection from without a toolchain.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Preset {
    /// Stable key, and what a caller passes back to create from it.
    pub id: &'static str,
    /// What the reader sees in the picker.
    pub display_name: &'static str,
    /// What this schema is for, in one sentence.
    pub description: &'static str,
    /// The named vector field a collection of this shape carries.
    pub field: &'static str,
}

/// Every preset, in the order a picker should show them.
pub const PRESETS: &[Preset] = &[
    Preset {
        id: "notes",
        display_name: "Notes",
        description: "Text with a title and tags. The smallest schema that is still a real one.",
        field: "text",
    },
    Preset {
        id: "memory",
        display_name: "Conversation memory",
        description: "Chat turns with a role and a conversation id, in the shape agent \
                      frameworks already use.",
        field: "text",
    },
];

/// The compiled descriptor set for `id`, if there is one.
///
/// `include_bytes!` rather than a file read: the bytes are part of the binary,
/// so a preset cannot go missing from an installed app because a file was not
/// packaged with it.
fn descriptor_set(id: &str) -> Option<&'static [u8]> {
    match id {
        "notes" => Some(include_bytes!("../../../presets/notes.binpb")),
        "memory" => Some(include_bytes!("../../../presets/memory.binpb")),
        _ => None,
    }
}

/// Build the create request for `preset_id` and `collection_id`.
///
/// Returns `None` for an unknown preset rather than defaulting to one — a typo
/// should not quietly create a collection with a schema the caller did not ask
/// for, because the schema is permanent once points are written under it.
pub fn to_new_collection(preset_id: &str, collection_id: &str) -> Option<NewCollection> {
    let preset = PRESETS.iter().find(|p| p.id == preset_id)?;
    let bytes = descriptor_set(preset_id)?;
    Some(NewCollection::new(collection_id, bytes.to_vec()).field(
        preset.field,
        TEXT_DIMENSIONS,
        // Cosine, because it is what text embedding models are trained for.
        // L2 here would be a quiet accuracy loss rather than an error.
        Metric::Cosine,
    ))
}

#[cfg(test)]
#[path = "presets_test.rs"]
mod tests;
