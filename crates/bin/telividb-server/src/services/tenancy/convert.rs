//! Between the domain's tenancy types and the wire's.
//!
//! Kept in one file because the four conversions share every rule that matters
//! — how a lifecycle becomes four timestamps, how absence is spelled — and
//! splitting them would put those rules in four places to disagree.

use crate::services::clock::{maybe, stamp};
use telividb_buffers::protobuf::tenancy::v1 as wire;
use telividb_core::{Lifecycle, Organization, Project, Protection, Session, Space};

/// An organization on the wire.
pub(super) fn organization(value: &Organization) -> wire::Organization {
    wire::Organization {
        name: value.name.as_str().to_owned(),
        display_name: value.display_name.clone(),
        create_time: Some(stamp(value.lifecycle.created_at)),
        update_time: Some(stamp(value.lifecycle.updated_at)),
        delete_time: maybe(value.lifecycle.deleted_at),
        expire_time: maybe(value.lifecycle.expires_at),
        ..Default::default()
    }
}

/// A project on the wire.
pub(super) fn project(value: &Project) -> wire::Project {
    wire::Project {
        name: value.name.as_str().to_owned(),
        display_name: value.display_name.clone(),
        create_time: Some(stamp(value.lifecycle.created_at)),
        update_time: Some(stamp(value.lifecycle.updated_at)),
        delete_time: maybe(value.lifecycle.deleted_at),
        expire_time: maybe(value.lifecycle.expires_at),
        ..Default::default()
    }
}

/// A space on the wire.
pub(super) fn space(value: &Space) -> wire::Space {
    wire::Space {
        name: value.name.as_str().to_owned(),
        display_name: value.display_name.clone(),
        projects: value
            .projects
            .iter()
            .map(|p| p.as_str().to_owned())
            .collect(),
        protection: protection(value.protection) as i32,
        // A key-wrapped space is locked until something unwraps it, and nothing
        // does yet. Reported as locked rather than as readable, because a
        // caller told it is open would believe it had been decrypted.
        locked: matches!(value.protection, Protection::Vault | Protection::Sealed),
        create_time: Some(stamp(value.lifecycle.created_at)),
        update_time: Some(stamp(value.lifecycle.updated_at)),
        delete_time: maybe(value.lifecycle.deleted_at),
        expire_time: maybe(value.lifecycle.expires_at),
        ..Default::default()
    }
}

/// A session on the wire.
pub(super) fn session(value: &Session) -> wire::Session {
    wire::Session {
        name: value.name.as_str().to_owned(),
        display_name: value.display_name.clone(),
        space: value
            .space
            .as_ref()
            .map(|s| s.as_str().to_owned())
            .unwrap_or_default(),
        create_time: Some(stamp(value.lifecycle.created_at)),
        update_time: Some(stamp(value.lifecycle.updated_at)),
        delete_time: maybe(value.lifecycle.deleted_at),
        expire_time: maybe(value.lifecycle.expires_at),
        ..Default::default()
    }
}

/// The wire value for a protection state.
pub(super) fn protection(value: Protection) -> wire::Protection {
    match value {
        Protection::None => wire::Protection::None,
        Protection::Private => wire::Protection::Private,
        Protection::Vault => wire::Protection::Vault,
        Protection::Sealed => wire::Protection::Sealed,
    }
}

/// The domain value for a wire protection state.
///
/// Unspecified means the caller did not choose, and the least protective
/// option is the right default for a *new* space: someone who wanted a vault
/// says so, and a space silently created as one could not be read back by the
/// person who made it.
///
/// Note this is the opposite direction from the storage decoder, which reads an
/// unknown protection as `Sealed`. The cases differ: there a record already
/// exists and may hold protected data, here nothing has been written yet.
pub(super) fn from_wire_protection(value: i32) -> Protection {
    match wire::Protection::try_from(value) {
        Ok(wire::Protection::Private) => Protection::Private,
        Ok(wire::Protection::Vault) => Protection::Vault,
        Ok(wire::Protection::Sealed) => Protection::Sealed,
        _ => Protection::None,
    }
}

/// A lifecycle for something created now.
pub(super) fn born(now_millis: i64) -> Lifecycle {
    Lifecycle {
        created_at: now_millis,
        updated_at: now_millis,
        deleted_at: None,
        expires_at: None,
    }
}
