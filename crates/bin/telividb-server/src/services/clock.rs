//! Wall-clock time, in the two shapes the services need.
//!
//! Shared rather than written per service. Two definitions of "now" is how a
//! `create_time` and an `update_time` come to disagree about which epoch they
//! are in — and a wrong timestamp is not a failure anything reports.

/// Milliseconds since the Unix epoch.
///
/// Zero if the clock is before the epoch, which is not a state worth an error:
/// a machine set that badly has larger problems, and refusing to serve is a
/// worse answer than an obviously wrong timestamp.
pub(super) fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

/// Milliseconds as a protobuf timestamp.
pub(super) fn stamp(millis: i64) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: millis / 1_000,
        nanos: ((millis % 1_000) * 1_000_000) as i32,
    }
}

/// An optional time, absent when there is none.
///
/// `None` rather than a zero timestamp: on the wire an unset field is how
/// "never deleted" is said, and a zero would read as the epoch — a real time,
/// and a wrong one.
pub(super) fn maybe(millis: Option<i64>) -> Option<prost_types::Timestamp> {
    millis.map(stamp)
}

/// Now, as a protobuf timestamp.
pub(super) fn now_stamp() -> prost_types::Timestamp {
    stamp(now_millis())
}
