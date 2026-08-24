//! Encoding one collection's catalogue entry as `redb` value bytes.
//!
//! Versioned like every other on-disk structure (rule 4): the leading byte is
//! refused rather than guessed at if a newer telividb wrote it. Without that,
//! a future encoding would be read as this one and silently produce a
//! collection declaring the wrong widths — which is exactly the failure the
//! declaration exists to prevent.

use telividb_core::{Collection, Dim, Error, Fingerprint, Metric, Result, VectorFieldSpec};
use telividb_core::{IndexKind, ResourceName};

/// Record version. Bump when the layout below changes.
const VERSION: u8 = 1;

/// Serialize a catalogue entry.
///
/// The name comes from the redb key and is never written into the value — one
/// identity, in one place, so the two cannot disagree.
pub(super) fn encode(collection: &Collection, descriptor_set: &[u8]) -> Vec<u8> {
    let mut out = vec![VERSION];
    out.extend_from_slice(collection.fingerprint.as_bytes());
    push_bytes(&mut out, descriptor_set);

    out.extend_from_slice(&(collection.vector_fields.len() as u32).to_le_bytes());
    for field in &collection.vector_fields {
        push_bytes(&mut out, field.name.as_bytes());
        out.extend_from_slice(&(field.dim.get() as u32).to_le_bytes());
        out.push(metric_byte(field.metric));
        out.push(index_byte(field.index));
        push_bytes(&mut out, field.model.as_bytes());
        out.extend_from_slice(field.model_fingerprint.as_bytes());
        push_bytes(&mut out, field.query_encoder.as_deref().unwrap_or("").as_bytes());
        push_bytes(&mut out, field.permission.as_deref().unwrap_or("").as_bytes());
    }
    out
}

/// Read a catalogue entry back, with `name` supplied by the caller.
pub(super) fn decode(name: ResourceName, bytes: &[u8]) -> Result<(Collection, Vec<u8>)> {
    let mut cursor = Cursor { bytes, offset: 0 };
    let version = cursor.byte()?;
    if version != VERSION {
        return Err(Error::PointStore {
            reason: format!(
                "collection record version {version} is not readable by this \
                 build, which writes version {VERSION}"
            ),
        });
    }

    let fingerprint = Fingerprint::from_bytes(cursor.array32()?);
    let descriptor_set = cursor.bytes()?.to_vec();

    let count = cursor.u32()? as usize;
    let mut fields = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let name = cursor.string()?;
        let dim = Dim::new(cursor.u32()?)?;
        let metric = metric_from(cursor.byte()?)?;
        let index = index_from(cursor.byte()?)?;
        let model = cursor.string()?;
        let model_fingerprint = Fingerprint::from_bytes(cursor.array32()?);
        let query_encoder = cursor.string()?;
        let permission = cursor.string()?;

        fields.push(VectorFieldSpec {
            name,
            dim,
            metric,
            index,
            model,
            model_fingerprint,
            query_encoder: none_if_empty(query_encoder),
            permission: none_if_empty(permission),
        });
    }

    Ok((
        Collection {
            name,
            fingerprint,
            vector_fields: fields,
        },
        descriptor_set,
    ))
}

/// An empty string means "not declared", not a field named `""`.
fn none_if_empty(value: String) -> Option<String> {
    match value.is_empty() {
        true => None,
        false => Some(value),
    }
}

/// Length-prefixed bytes.
fn push_bytes(out: &mut Vec<u8>, value: &[u8]) {
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value);
}

/// Stable on-disk discriminants. Never renumber: a stored record names its
/// metric by this byte, so a change reinterprets every collection written
/// before it.
fn metric_byte(metric: Metric) -> u8 {
    match metric {
        Metric::Dot => 0,
        Metric::L2 => 1,
        Metric::Cosine => 2,
    }
}

/// The inverse of [`metric_byte`].
fn metric_from(value: u8) -> Result<Metric> {
    match value {
        0 => Ok(Metric::Dot),
        1 => Ok(Metric::L2),
        2 => Ok(Metric::Cosine),
        other => Err(Error::PointStore {
            reason: format!("unknown metric discriminant {other}"),
        }),
    }
}

/// Stable on-disk discriminants for the index kind.
fn index_byte(index: IndexKind) -> u8 {
    match index {
        IndexKind::Flat => 0,
        IndexKind::Hnsw => 1,
        IndexKind::IvfPq => 2,
    }
}

/// The inverse of [`index_byte`].
fn index_from(value: u8) -> Result<IndexKind> {
    match value {
        0 => Ok(IndexKind::Flat),
        1 => Ok(IndexKind::Hnsw),
        2 => Ok(IndexKind::IvfPq),
        other => Err(Error::PointStore {
            reason: format!("unknown index discriminant {other}"),
        }),
    }
}

/// A bounds-checked reader over the record.
///
/// Every read is checked because these bytes come from disk, where a truncated
/// write is a real outcome — an unchecked slice would panic inside a request
/// handler rather than report a corrupt record.
struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Cursor<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self.offset.checked_add(n).ok_or_else(|| truncated(n))?;
        let slice = self.bytes.get(self.offset..end).ok_or_else(|| truncated(n))?;
        self.offset = end;
        Ok(slice)
    }

    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32> {
        let raw = self.take(4)?;
        Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    fn array32(&mut self) -> Result<[u8; 32]> {
        let raw = self.take(32)?;
        let mut out = [0u8; 32];
        out.copy_from_slice(raw);
        Ok(out)
    }

    fn bytes(&mut self) -> Result<&[u8]> {
        let len = self.u32()? as usize;
        self.take(len)
    }

    fn string(&mut self) -> Result<String> {
        let raw = self.bytes()?;
        String::from_utf8(raw.to_vec()).map_err(|e| Error::PointStore {
            reason: format!("collection record holds invalid utf-8: {e}"),
        })
    }
}

/// The "ran off the end" error, shared by every read.
fn truncated(needed: usize) -> Error {
    Error::PointStore {
        reason: format!("collection record is truncated: needed {needed} more bytes"),
    }
}
