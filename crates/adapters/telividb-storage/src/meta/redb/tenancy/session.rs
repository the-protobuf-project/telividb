//! Encoding one session as `redb` value bytes, through the schema.

use super::time;
use crate::error::Result;
use capnp::message::{Builder, HeapAllocator, ReaderOptions};
use capnp::serialize;
use telividb_buffers::capnp::tenancy::v1::session_capnp::session;
use telividb_core::{Error as DomainError, ResourceName, Session};

/// Serialize a session.
pub(super) fn encode(value: &Session) -> Vec<u8> {
    let mut message = Builder::new(HeapAllocator::new());
    {
        let mut root = message.init_root::<session::Builder<'_>>();
        root.set_display_name(value.display_name.as_str());
        // An absent space is the empty string rather than a missing field:
        // Cap'n Proto text has no null, and "" is not a resource name, so the
        // two cannot be confused.
        root.set_space(value.space.as_ref().map(|s| s.as_str()).unwrap_or_default());
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

/// Read a session back, with `name` supplied by the caller.
pub(super) fn decode(name: ResourceName, bytes: &[u8]) -> Result<Session> {
    let mut cursor = std::io::Cursor::new(bytes);
    let message = serialize::read_message(&mut cursor, ReaderOptions::new()).map_err(malformed)?;
    let root = message
        .get_root::<session::Reader<'_>>()
        .map_err(malformed)?;

    let space = root
        .get_space()
        .map_err(malformed)?
        .to_str()
        .map_err(|_| malformed_str("space is not valid UTF-8"))?
        .to_owned();

    Ok(Session {
        name,
        display_name: root
            .get_display_name()
            .map_err(malformed)?
            .to_str()
            .map_err(|_| malformed_str("display_name is not valid UTF-8"))?
            .to_owned(),
        space: match space.is_empty() {
            true => None,
            false => Some(ResourceName::parse(&space).map_err(|e| malformed_str(&e.to_string()))?),
        },
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
        reason: format!("malformed session record: {reason}"),
    }
    .into()
}
