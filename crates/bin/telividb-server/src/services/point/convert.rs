//! Converting between the wire `Point` and `telividb_core::Point`.
//!
//! Split out of `point.rs` so the RPC handlers read as request handling, not
//! field-by-field mapping.

use super::super::vector::convert::{vector_to_domain, vector_to_wire};
use telividb_buffers::protobuf::point::v1::{
    ContentRef as WireContentRef, NamedVector, Point as WirePoint, Span as WireSpan,
    Vector as WireVector,
};
use telividb_core::{ContentRef, ResourceName, Span};
use tonic::Status;

/// Build a domain `Point` from a wire `Point` and the name it will be
/// created under.
pub(super) fn to_domain(
    name: ResourceName,
    wire: WirePoint,
) -> Result<telividb_core::Point, Status> {
    let mut point = telividb_core::Point::new(name);
    point.span = wire.span.map(span_to_domain).transpose()?;
    point.content_ref = wire.content_ref.map(content_ref_to_domain).transpose()?;
    for named in wire.vectors {
        let field = named.field_id;
        if field.is_empty() {
            return Err(Status::invalid_argument(
                "a named vector needs a field_id: each field has its own model \
                 and metric, so a vector without one cannot be stored",
            ));
        }
        let vector = named
            .vector
            .ok_or_else(|| Status::invalid_argument(format!("vector for {field:?} is missing")))?;
        let decoded = vector_to_domain(&vector)?;
        // A repeat would silently overwrite the earlier vector, so the caller
        // would believe both were stored.
        if point.vectors.insert(field.clone(), decoded).is_some() {
            return Err(Status::invalid_argument(format!(
                "field_id {field:?} appears more than once"
            )));
        }
    }
    Ok(point)
}

/// Decode a wire vector's raw little-endian `f32` payload.
///
/// The bytes are carried as a length-delimited field rather than a repeated
/// scalar because protobuf encodes the latter element by element — 768 varint
/// operations per message on the hot path.
pub(super) 

/// The reverse of [`to_domain`], for responses.
pub(super) fn to_wire(point: telividb_core::Point) -> WirePoint {
    WirePoint {
        name: point.name.as_str().to_owned(),
        vectors: point
            .vectors
            .iter()
            .map(|(field, vector)| NamedVector {
                // Always empty on the way out: a stored point holds a vector,
                // and echoing the text back as though it were still pending
                // would invite a reader to embed it a second time.
                text: String::new(),
                field_id: field.clone(),
                vector: Some(vector_to_wire(vector)),
            })
            .collect(),
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
    // Checked: a large `seconds` would otherwise wrap and yield a span that
    // is silently wrong rather than refused.
    (d.seconds as u64)
        .checked_mul(1000)
        .and_then(|ms| ms.checked_add(d.nanos as u64 / 1_000_000))
        .ok_or_else(|| Status::invalid_argument("span offset overflows milliseconds"))
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
fn content_ref_to_domain(wire: WireContentRef) -> Result<ContentRef, Status> {
    if wire.range_start < 0 || wire.range_end < 0 {
        return Err(Status::invalid_argument(
            "a content range must not be negative",
        ));
    }
    // A digest of the wrong length is refused rather than dropped. Silently
    // discarding it would leave a point that looks unhashed, and stale-content
    // detection depends on the hash being there.
    let sha256 = if wire.sha256.is_empty() {
        None
    } else {
        let bytes: [u8; 32] = wire.sha256.as_ref().try_into().map_err(|_| {
            Status::invalid_argument(format!(
                "sha256 must be 32 bytes, got {}",
                wire.sha256.len()
            ))
        })?;
        Some(bytes)
    };
    Ok(ContentRef {
        uri: wire.uri,
        byte_range: (wire.range_start != 0 || wire.range_end != 0)
            .then_some((wire.range_start as u64, wire.range_end as u64)),
        sha256,
        inline: (!wire.inline_text.is_empty()).then_some(wire.inline_text),
    })
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
