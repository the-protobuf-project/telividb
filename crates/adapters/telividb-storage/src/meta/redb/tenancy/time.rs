//! Timestamps, between the domain and the schema.
//!
//! Shared by every record in this module because all four resources carry the
//! same four times, and four copies of this would be four places to get the
//! zero-means-absent rule wrong.

use telividb_buffers::capnp::buffers::wellknown_capnp::timestamp;
use telividb_core::Lifecycle;

/// Write milliseconds into a timestamp, leaving it zero when there is none.
///
/// A zero timestamp reads back as `None`, which is how an absent `delete_time`
/// survives the round trip — Cap'n Proto has no null for a struct field, so
/// absence has to be a value the schema can hold. Zero is safe to spend on it:
/// the epoch is not a time any of these resources was created.
pub(super) fn write(mut out: timestamp::Builder<'_>, millis: Option<i64>) {
    let Some(millis) = millis else { return };
    out.set_seconds(millis / 1_000);
    out.set_nanos(((millis % 1_000) * 1_000_000) as i32);
}

/// Read a timestamp back to milliseconds, treating zero as absent.
pub(super) fn read(time: timestamp::Reader<'_>) -> Option<i64> {
    let seconds = time.get_seconds();
    let nanos = i64::from(time.get_nanos());
    (seconds != 0 || nanos != 0).then(|| seconds * 1_000 + nanos / 1_000_000)
}

/// The four times a lifecycle carries, read from whatever holds them.
///
/// Takes the four readers rather than the record, because Cap'n Proto generates
/// a separate type per message and there is no trait tying them together — so a
/// generic version would need one written by hand for no gain.
pub(super) fn lifecycle(
    create: timestamp::Reader<'_>,
    update: timestamp::Reader<'_>,
    delete: timestamp::Reader<'_>,
    expire: timestamp::Reader<'_>,
) -> Lifecycle {
    Lifecycle {
        created_at: read(create).unwrap_or_default(),
        updated_at: read(update).unwrap_or_default(),
        deleted_at: read(delete),
        expires_at: read(expire),
    }
}
