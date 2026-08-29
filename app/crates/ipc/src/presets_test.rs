//! Tests for the shipped presets.
//!
//! These run without a window and without an engine. What they check is that
//! the bytes in the binary are real descriptor sets rather than whatever
//! happened to be on disk when someone last ran the generator.

use super::*;

#[test]
fn every_preset_carries_a_descriptor_set() {
    for preset in PRESETS {
        let bytes = descriptor_set(preset.id)
            .unwrap_or_else(|| panic!("{} has no descriptor set", preset.id));
        assert!(
            !bytes.is_empty(),
            "{} has an empty descriptor set",
            preset.id
        );

        // A `FileDescriptorSet` opens with field 1, wire type 2 — a
        // length-delimited `FileDescriptorProto`. Byte `0x0a` is that tag.
        // Cheap, and enough to catch bytes that are not a descriptor set at
        // all, which is the failure a `include_bytes!` of the wrong path gives.
        assert_eq!(
            bytes.first(),
            Some(&0x0a),
            "{} does not begin like a FileDescriptorSet",
            preset.id
        );
    }
}

#[test]
fn every_preset_builds_a_collection() {
    // The id a caller passes is the collection's, not the preset's — one preset
    // creates as many collections as a person wants. `NewCollection` exposes no
    // getters, so this asserts only that the build succeeds for every preset,
    // which is what a picker depends on.
    for preset in PRESETS {
        assert!(
            to_new_collection(preset.id, "anything").is_some(),
            "{} did not build a collection",
            preset.id
        );
    }
}

#[test]
fn an_unknown_preset_is_refused_rather_than_defaulted() {
    // A schema is permanent once points are written under it, so a typo must
    // not quietly produce a collection shaped like something else.
    assert!(to_new_collection("notez", "oops").is_none());
}
