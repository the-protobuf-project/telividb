//! Tests for the typed resource keys.
//!
//! The interesting cases are the refusals. Generating a well-formed name is
//! easy to get right by hand; noticing that a name has the wrong *shape* is
//! what string surgery gets wrong, and what these keys exist to prevent.

use super::*;

#[test]
fn an_organization_name_round_trips() {
    let key = OrganizationKey {
        organization: "acme".to_owned(),
    };
    let name = key.generate().expect("generates");
    assert_eq!(name, "organizations/acme");
    assert_eq!(OrganizationKey::parse(&name).expect("parses"), key);
}

#[test]
fn a_nested_name_round_trips_with_every_segment() {
    let key = MessageKey {
        organization: "acme".to_owned(),
        conversation: "c-1".to_owned(),
        message: "m-9".to_owned(),
    };
    assert_eq!(
        key.generate().expect("generates"),
        "organizations/acme/conversations/c-1/messages/m-9"
    );
    assert_eq!(
        MessageKey::parse("organizations/acme/conversations/c-1/messages/m-9").expect("parses"),
        key
    );
}

#[test]
fn a_project_name_is_not_an_organization_name() {
    // The failure this replaces: `name.split('/')` reads the first two segments
    // and returns `acme`, silently treating a project as its parent. The
    // template refuses because the shape does not match.
    assert!(
        OrganizationKey::parse("organizations/acme/projects/atlas").is_err(),
        "a project name parsed as an organization"
    );
}

#[test]
fn a_truncated_name_is_refused() {
    assert!(ProjectKey::parse("organizations/acme").is_err());
    assert!(ProjectKey::parse("organizations/acme/projects").is_err());
}

#[test]
fn the_collection_prefix_is_not_interchangeable() {
    // Two resources whose names differ only in a literal segment. Nothing about
    // the string length or the segment count separates them — only the
    // template does.
    assert!(OrganizationKey::parse("collections/acme").is_err());
}

#[test]
fn an_id_containing_a_slash_is_refused() {
    // A placeholder matches exactly one segment. Without that rule an id
    // carrying a slash would silently produce a name with more segments than
    // the template declares — which parses back as a different resource.
    let key = OrganizationKey {
        organization: "acme/evil".to_owned(),
    };
    assert!(key.generate().is_err(), "a slash escaped into a name");
}
