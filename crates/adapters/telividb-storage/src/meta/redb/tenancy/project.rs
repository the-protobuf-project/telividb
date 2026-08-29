//! Encoding one project as `redb` value bytes, through the schema.

use super::time;
use crate::error::Result;
use capnp::message::{Builder, HeapAllocator, ReaderOptions};
use capnp::serialize;
use telividb_buffers::capnp::tenancy::v1::project_capnp::project;
use telividb_core::{Error as DomainError, Project, ResourceName};

/// Serialize a project.
///
/// The name comes from the `redb` key and is never written into the value —
/// one identity, in one place, so the two cannot disagree.
pub(super) fn encode(value: &Project) -> Vec<u8> {
    let mut message = Builder::new(HeapAllocator::new());
    {
        let mut root = message.init_root::<project::Builder<'_>>();
        root.set_display_name(value.display_name.as_str());
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

/// Read a project back, with `name` supplied by the caller.
pub(super) fn decode(name: ResourceName, bytes: &[u8]) -> Result<Project> {
    // `read_message` rather than the flat-slice reader: a `redb` value is a
    // borrowed slice into a mapped page at whatever offset the record sits at,
    // and the zero-copy reader requires 8-byte alignment.
    let mut cursor = std::io::Cursor::new(bytes);
    let message = serialize::read_message(&mut cursor, ReaderOptions::new()).map_err(malformed)?;
    let root = message
        .get_root::<project::Reader<'_>>()
        .map_err(malformed)?;

    Ok(Project {
        name,
        display_name: root
            .get_display_name()
            .map_err(malformed)?
            .to_str()
            .map_err(|_| malformed_str("display_name is not valid UTF-8"))?
            .to_owned(),
        lifecycle: time::lifecycle(
            root.get_create_time().map_err(malformed)?,
            root.get_update_time().map_err(malformed)?,
            root.get_delete_time().map_err(malformed)?,
            root.get_expire_time().map_err(malformed)?,
        ),
    })
}

/// A record that could not be read as this schema.
fn malformed(error: capnp::Error) -> crate::error::Error {
    malformed_str(&error.to_string())
}

/// The same, from a message of our own.
fn malformed_str(reason: &str) -> crate::error::Error {
    DomainError::PointStore {
        reason: format!("malformed project record: {reason}"),
    }
    .into()
}
