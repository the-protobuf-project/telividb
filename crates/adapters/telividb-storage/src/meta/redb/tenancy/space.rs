//! Encoding one space as `redb` value bytes, through the schema.
//!
//! A space carries more than the others: the projects it belongs to, and how
//! its contents are protected. Protection is why the extra field matters —
//! it decides segment routing, so a record that lost it would put a vault's
//! points in with everyone else's.

use super::time;
use crate::error::Result;
use capnp::message::{Builder, HeapAllocator, ReaderOptions};
use capnp::serialize;
use telividb_buffers::capnp::tenancy::v1::space_capnp::{Protection as WireProtection, space};
use telividb_core::{Error as DomainError, Protection, ResourceName, Space};

/// Serialize a space.
pub(super) fn encode(value: &Space) -> Vec<u8> {
    let mut message = Builder::new(HeapAllocator::new());
    {
        let mut root = message.init_root::<space::Builder<'_>>();
        root.set_display_name(value.display_name.as_str());
        root.set_protection(to_wire(value.protection));

        let mut projects = root.reborrow().init_projects(value.projects.len() as u32);
        for (index, project) in value.projects.iter().enumerate() {
            // Typed explicitly: `.into()` here is ambiguous, because several
            // crates in this tree implement `From<&str>` for their own text.
            projects.set(index as u32, capnp::text::Reader::from(project.as_str()));
        }

        time::write(
            root.reborrow().init_create_time(),
            Some(value.lifecycle.created_at),
        );
        time::write(
            root.reborrow().init_update_time(),
            Some(value.lifecycle.updated_at),
        );
        time::write(
            root.reborrow().init_delete_time(),
            value.lifecycle.deleted_at,
        );
        time::write(
            root.reborrow().init_expire_time(),
            value.lifecycle.expires_at,
        );
    }
    serialize::write_message_to_words(&message)
}

/// Read a space back, with `name` supplied by the caller.
pub(super) fn decode(name: ResourceName, bytes: &[u8]) -> Result<Space> {
    let mut cursor = std::io::Cursor::new(bytes);
    let message = serialize::read_message(&mut cursor, ReaderOptions::new()).map_err(malformed)?;
    let root = message.get_root::<space::Reader<'_>>().map_err(malformed)?;

    let mut projects = Vec::new();
    for project in root.get_projects().map_err(malformed)?.iter() {
        let text = project
            .map_err(malformed)?
            .to_str()
            .map_err(|_| malformed_str("a project name is not valid UTF-8"))?
            .to_owned();
        projects.push(ResourceName::parse(&text).map_err(|e| malformed_str(&e.to_string()))?);
    }

    Ok(Space {
        name,
        display_name: root
            .get_display_name()
            .map_err(malformed)?
            .to_str()
            .map_err(|_| malformed_str("display_name is not valid UTF-8"))?
            .to_owned(),
        projects,
        // A value this build does not know is not an error — a newer
        // telividb may have written a protection state that did not exist
        // when this one was compiled. It reads as the most protective option
        // available rather than the least, for the reason in `from_wire`.
        protection: root
            .get_protection()
            .map(from_wire)
            .unwrap_or(Protection::Sealed),
        lifecycle: time::lifecycle(
            root.get_create_time().map_err(malformed)?,
            root.get_update_time().map_err(malformed)?,
            root.get_delete_time().map_err(malformed)?,
            root.get_expire_time().map_err(malformed)?,
        ),
    })
}

/// The wire value for a protection state.
fn to_wire(protection: Protection) -> WireProtection {
    match protection {
        Protection::None => WireProtection::None,
        Protection::Private => WireProtection::Private,
        Protection::Vault => WireProtection::Vault,
        Protection::Sealed => WireProtection::Sealed,
    }
}

/// The domain value for a wire protection state.
///
/// An unspecified value reads as the *most* protective option this build knows,
/// not the least — and so does a value it cannot name at all, handled by the
/// caller. A record written by a newer telividb carrying an unfamiliar
/// protection must not be treated as public: failing closed is the only safe
/// direction, and the cost of being wrong is a space that looks locked when it
/// is not, rather than one that looks open when it should not be.
fn from_wire(protection: WireProtection) -> Protection {
    match protection {
        WireProtection::None => Protection::None,
        WireProtection::Private => Protection::Private,
        WireProtection::Vault => Protection::Vault,
        WireProtection::Sealed => Protection::Sealed,
        WireProtection::Unspecified => Protection::Sealed,
    }
}

/// A record that could not be read as this schema.
fn malformed(error: capnp::Error) -> crate::error::Error {
    malformed_str(&error.to_string())
}

/// The same, from a message of our own.
fn malformed_str(reason: &str) -> crate::error::Error {
    DomainError::PointStore {
        reason: format!("malformed space record: {reason}"),
    }
    .into()
}
