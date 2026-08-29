//! Encoding one organization as `redb` value bytes, through the schema.
//!
//! # Why this is not another hand-written layout
//!
//! `collection_record.rs` writes its own bytes: a version prefix, lengths
//! pushed by hand, fields read back in the order they went out. It is careful,
//! and it is the kind of code that is wrong exactly once and silently — add a
//! field to the struct, forget it in `decode`, and every later field is read
//! from the wrong offset.
//!
//! Here the schema is the layout. It is generated from the same `.proto` the
//! gRPC surface uses, so a stored organization and a wire organization cannot
//! drift into two definitions that must agree. Cap'n Proto's own evolution
//! rules take the place of the version byte: a field added later reads as its
//! default in an older record rather than as whatever bytes follow.
//!
//! `buffers.lock` is what makes that safe. It pins the ordinal every field was
//! assigned, so a rebuild that would move one fails CI rather than silently
//! reinterpreting every record already on disk.

use crate::error::Result;
use capnp::message::{Builder, HeapAllocator, ReaderOptions};
use capnp::serialize;
use telividb_buffers::capnp::tenancy::v1::organization_capnp::organization;
use telividb_core::{Error as DomainError, Lifecycle, Organization, ResourceName};

/// Serialize an organization.
///
/// The name comes from the `redb` key and is never written into the value —
/// one identity, in one place, so the two cannot disagree.
pub(super) fn encode(org: &Organization) -> Vec<u8> {
    let mut message = Builder::new(HeapAllocator::new());
    {
        let mut root = message.init_root::<organization::Builder<'_>>();
        root.set_display_name(org.display_name.as_str());
        write_time(
            root.reborrow().init_create_time(),
            Some(org.lifecycle.created_at),
        );
        write_time(
            root.reborrow().init_update_time(),
            Some(org.lifecycle.updated_at),
        );
        write_time(root.reborrow().init_delete_time(), org.lifecycle.deleted_at);
        write_time(root.reborrow().init_expire_time(), org.lifecycle.expires_at);
    }
    serialize::write_message_to_words(&message)
}

/// Read an organization back, with `name` supplied by the caller.
pub(super) fn decode(name: ResourceName, bytes: &[u8]) -> Result<Organization> {
    // `read_message` rather than `read_message_from_flat_slice`.
    //
    // The flat-slice reader is zero-copy and requires the bytes to be 8-byte
    // aligned. A `redb` value is a borrowed slice into a memory-mapped page at
    // whatever offset the record happens to sit, so that requirement does not
    // hold — and the failure is invisible in a test that encodes and decodes
    // through a `Vec`, which is always aligned. This copies into an aligned
    // buffer, which for a metadata record is a few hundred bytes.
    let mut cursor = std::io::Cursor::new(bytes);
    let message = serialize::read_message(&mut cursor, ReaderOptions::new()).map_err(malformed)?;
    let root = message
        .get_root::<organization::Reader<'_>>()
        .map_err(malformed)?;

    Ok(Organization {
        name,
        display_name: root
            .get_display_name()
            .map_err(malformed)?
            .to_str()
            .map_err(|_| malformed_str("display_name is not valid UTF-8"))?
            .to_owned(),
        lifecycle: Lifecycle {
            created_at: read_time(root.get_create_time().map_err(malformed)?).unwrap_or_default(),
            updated_at: read_time(root.get_update_time().map_err(malformed)?).unwrap_or_default(),
            deleted_at: read_time(root.get_delete_time().map_err(malformed)?),
            expires_at: read_time(root.get_expire_time().map_err(malformed)?),
        },
    })
}

/// Write milliseconds into a timestamp, leaving it zero when there is none.
///
/// A zero timestamp reads back as `None`, which is how an absent `delete_time`
/// survives the round trip — Cap'n Proto has no null for a struct field, so
/// absence has to be a value the schema can hold.
fn write_time(
    mut out: telividb_buffers::capnp::buffers::wellknown_capnp::timestamp::Builder<'_>,
    millis: Option<i64>,
) {
    let Some(millis) = millis else { return };
    out.set_seconds(millis / 1_000);
    out.set_nanos(((millis % 1_000) * 1_000_000) as i32);
}

/// Read a timestamp back to milliseconds, treating zero as absent.
fn read_time(
    time: telividb_buffers::capnp::buffers::wellknown_capnp::timestamp::Reader<'_>,
) -> Option<i64> {
    let seconds = time.get_seconds();
    let nanos = i64::from(time.get_nanos());
    (seconds != 0 || nanos != 0).then(|| seconds * 1_000 + nanos / 1_000_000)
}

/// A record that could not be read as this schema.
fn malformed(error: capnp::Error) -> crate::error::Error {
    malformed_str(&error.to_string())
}

/// The same, from a message of our own.
fn malformed_str(reason: &str) -> crate::error::Error {
    DomainError::PointStore {
        reason: format!("malformed organization record: {reason}"),
    }
    .into()
}

#[cfg(test)]
#[path = "record_test.rs"]
mod tests;
