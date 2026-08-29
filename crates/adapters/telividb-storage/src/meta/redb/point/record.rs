//! Encoding and decoding one point's `span`/`content_ref` as `redb` value
//! bytes. Split out of `redb_point_store.rs` — the store's read/write
//! surface and this record format are two different concerns bundled behind
//! one file only by which crate they live in.

use telividb_core::{ContentRef, Error, Point, ResourceName, Result, Span};

const HAS_SPAN: u8 = 1 << 0;
const HAS_CONTENT_REF: u8 = 1 << 1;
const HAS_BYTE_RANGE: u8 = 1 << 2;
const HAS_SHA256: u8 = 1 << 3;
const HAS_INLINE: u8 = 1 << 4;

/// A presence-flags byte, then each present field in a fixed order: `span`,
/// then `content_ref`'s `uri`, `byte_range`, `sha256`, `inline`.
pub(super) fn encode(point: &Point) -> Vec<u8> {
    let mut out = Vec::new();
    let mut flags = 0u8;
    if point.span.is_some() {
        flags |= HAS_SPAN;
    }
    if let Some(content_ref) = &point.content_ref {
        flags |= HAS_CONTENT_REF;
        flags |= HAS_BYTE_RANGE * content_ref.byte_range.is_some() as u8;
        flags |= HAS_SHA256 * content_ref.sha256.is_some() as u8;
        flags |= HAS_INLINE * content_ref.inline.is_some() as u8;
    }
    out.push(flags);

    if let Some(span) = point.span {
        out.extend_from_slice(&span.start_ms().to_le_bytes());
        out.extend_from_slice(&span.end_ms().to_le_bytes());
    }
    if let Some(content_ref) = &point.content_ref {
        push_string(&mut out, &content_ref.uri);
        if let Some((start, end)) = content_ref.byte_range {
            out.extend_from_slice(&start.to_le_bytes());
            out.extend_from_slice(&end.to_le_bytes());
        }
        if let Some(sha256) = &content_ref.sha256 {
            out.extend_from_slice(sha256);
        }
        if let Some(inline) = &content_ref.inline {
            push_string(&mut out, inline);
        }
    }
    out
}

fn push_string(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// The inverse of [`encode`]. `name` comes from the redb key, not the value —
/// a point's own name is never re-encoded into its record.
pub(super) fn decode(name: ResourceName, bytes: &[u8]) -> Result<Point> {
    let mut cursor = Cursor { bytes, offset: 0 };
    let flags = cursor.byte()?;

    let span = (flags & HAS_SPAN != 0)
        .then(|| {
            let start = cursor.u64()?;
            let end = cursor.u64()?;
            Span::new(start, end).map_err(|e| point_err(e.to_string()))
        })
        .transpose()?;

    let content_ref = if flags & HAS_CONTENT_REF != 0 {
        let uri = cursor.string()?;
        let byte_range = (flags & HAS_BYTE_RANGE != 0)
            .then(|| Ok::<_, Error>((cursor.u64()?, cursor.u64()?)))
            .transpose()?;
        let sha256 = (flags & HAS_SHA256 != 0)
            .then(|| cursor.bytes32())
            .transpose()?;
        let inline = (flags & HAS_INLINE != 0)
            .then(|| cursor.string())
            .transpose()?;
        Some(ContentRef {
            uri,
            byte_range,
            sha256,
            inline,
        })
    } else {
        None
    };

    let mut point = Point::new(name);
    point.span = span;
    point.content_ref = content_ref;
    Ok(point)
}

fn point_err(reason: String) -> Error {
    Error::PointStore { reason }
}

/// A cursor over a point record's encoded bytes, reporting truncation as a
/// `PointStore` error rather than panicking on a short read.
struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Cursor<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self.offset + n;
        let slice = self.bytes.get(self.offset..end).ok_or_else(|| {
            point_err(format!(
                "point record truncated: needed {n} bytes at offset {}, had {}",
                self.offset,
                self.bytes.len()
            ))
        })?;
        self.offset = end;
        Ok(slice)
    }

    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn bytes32(&mut self) -> Result<[u8; 32]> {
        Ok(self.take(32)?.try_into().unwrap())
    }

    fn string(&mut self) -> Result<String> {
        let len = u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize;
        String::from_utf8(self.take(len)?.to_vec()).map_err(|e| point_err(format!("{e}")))
    }
}

#[cfg(test)]
#[path = "record_test.rs"]
mod tests;
