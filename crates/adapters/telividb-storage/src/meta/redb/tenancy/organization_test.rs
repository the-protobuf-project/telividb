//! Round-trip tests for the organization record.
//!
//! The point of the schema-backed encoding is that a field cannot be written
//! and forgotten on the way back, so these assert on the whole value rather
//! than field by field — a comparison that keeps working when a field is added.

use super::*;
// `Lifecycle` left the record module when the time helpers were shared.
use telividb_core::Lifecycle;

/// An organization with every field populated.
fn full() -> Organization {
    Organization {
        name: ResourceName::parse("organizations/acme").expect("a valid name"),
        display_name: "Acme".to_owned(),
        lifecycle: Lifecycle {
            created_at: 1_700_000_000_123,
            updated_at: 1_700_000_100_456,
            deleted_at: Some(1_700_000_200_789),
            expires_at: Some(1_800_000_000_000),
        },
    }
}

#[test]
fn a_full_record_survives_the_round_trip() {
    let original = full();
    let decoded = decode(original.name.clone(), &encode(&original)).expect("decodes");
    assert_eq!(decoded, original);
}

#[test]
fn a_live_organization_has_no_delete_time() {
    // The common case, and the one a hand-written layout gets wrong: Cap'n
    // Proto has no null for a struct field, so absence has to survive as a
    // value the schema can hold.
    let mut original = full();
    original.lifecycle.deleted_at = None;
    original.lifecycle.expires_at = None;

    let decoded = decode(original.name.clone(), &encode(&original)).expect("decodes");
    assert_eq!(decoded.lifecycle.deleted_at, None);
    assert_eq!(decoded.lifecycle.expires_at, None);
    assert!(decoded.lifecycle.is_live());
}

#[test]
fn the_name_comes_from_the_key_not_the_value() {
    // One identity, in one place. The encoder never writes the name, so a
    // record read under a different key takes that key — which is what makes
    // the two impossible to disagree.
    let original = full();
    let other = ResourceName::parse("organizations/other").expect("a valid name");
    let decoded = decode(other.clone(), &encode(&original)).expect("decodes");
    assert_eq!(decoded.name, other);
}

#[test]
fn a_truncated_record_is_refused_rather_than_guessed_at() {
    let bytes = encode(&full());
    let truncated = &bytes[..bytes.len() / 2];
    assert!(
        decode(full().name, truncated).is_err(),
        "half a record decoded as a whole one"
    );
}
