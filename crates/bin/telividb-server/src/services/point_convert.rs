//! Converting between the wire `Point` and `telividb_core::Point`.
//!
//! Split out of `point.rs` so the RPC handlers read as request handling, not
//! field-by-field mapping.

use telividb_core::{ContentRef, ResourceName, Span};
use telividb_proto::point::v1::{
    ContentRef as WireContentRef, Point as WirePoint, Span as WireSpan,
};
use tonic::Status;

/// Build a domain `Point` from a wire `Point` and the name it will be
/// created under.
///
/// Refuses named vectors outright rather than silently dropping them: the
/// caller would otherwise believe they were stored. Vectors arrive with the
/// vector service.
pub(super) fn to_domain(
    name: ResourceName,
    wire: WirePoint,
) -> Result<telividb_core::Point, Status> {
    if !wire.vectors.is_empty() {
        return Err(Status::unimplemented(
            "named vectors are not yet supported; they land with the vector service",
        ));
    }
    let mut point = telividb_core::Point::new(name);
    point.span = wire.span.map(span_to_domain).transpose()?;
    point.content_ref = wire.content_ref.map(content_ref_to_domain);
    Ok(point)
}

/// The reverse of [`to_domain`], for responses.
pub(super) fn to_wire(point: telividb_core::Point) -> WirePoint {
    WirePoint {
        name: point.name.as_str().to_owned(),
        vectors: Vec::new(),
        span: point.span.map(span_to_wire),
        content_ref: point.content_ref.map(content_ref_to_wire),
    }
}

fn span_to_domain(wire: WireSpan) -> Result<Span, Status> {
    let start = wire
        .start_offset
        .ok_or_else(|| Status::invalid_argument("span.start_offset is required"))?;
    let end = wire
        .end_offset
        .ok_or_else(|| Status::invalid_argument("span.end_offset is required"))?;
    Span::new(duration_to_ms(&start)?, duration_to_ms(&end)?)
        .map_err(|e| Status::invalid_argument(e.to_string()))
}

fn span_to_wire(span: Span) -> WireSpan {
    WireSpan {
        start_offset: Some(ms_to_duration(span.start_ms())),
        end_offset: Some(ms_to_duration(span.end_ms())),
    }
}

fn duration_to_ms(d: &prost_types::Duration) -> Result<u64, Status> {
    if d.seconds < 0 || d.nanos < 0 {
        return Err(Status::invalid_argument(
            "a span offset must not be negative",
        ));
    }
    Ok(d.seconds as u64 * 1000 + d.nanos as u64 / 1_000_000)
}

fn ms_to_duration(ms: u64) -> prost_types::Duration {
    prost_types::Duration {
        seconds: (ms / 1000) as i64,
        nanos: ((ms % 1000) * 1_000_000) as i32,
    }
}

/// `range_start`/`range_end`/`sha256`/`inline_text` are plain proto3 scalars,
/// so "absent" is a convention (zero, empty) rather than a wire-level fact —
/// unlike everywhere else in this codebase, where presence is explicit. This
/// is the one place that convention is applied.
fn content_ref_to_domain(wire: WireContentRef) -> ContentRef {
    ContentRef {
        uri: wire.uri,
        byte_range: (wire.range_start != 0 || wire.range_end != 0)
            .then_some((wire.range_start as u64, wire.range_end as u64)),
        sha256: (!wire.sha256.is_empty())
            .then(|| wire.sha256.to_vec())
            .and_then(|v| v.try_into().ok()),
        inline: (!wire.inline_text.is_empty()).then_some(wire.inline_text),
    }
}

fn content_ref_to_wire(content_ref: ContentRef) -> WireContentRef {
    let (range_start, range_end) = content_ref.byte_range.unwrap_or_default();
    WireContentRef {
        uri: content_ref.uri,
        range_start: range_start as i64,
        range_end: range_end as i64,
        sha256: content_ref
            .sha256
            .map(|s| s.to_vec())
            .unwrap_or_default()
            .into(),
        inline_text: content_ref.inline.unwrap_or_default(),
    }
}

#[cfg(test)]
#[path = "point_convert_test.rs"]
mod tests;
