//! What a vector field is, recorded beside its data.
//!
//! A field's dimension and metric are properties of the field, not of whatever
//! vector happens to arrive first. Persisting them means opening an existing
//! field needs no guess — and a caller that guesses wrong is told so, instead
//! of watching every replayed record be silently rejected for the wrong width.
//!
//! Versioned with magic bytes like every other on-disk structure here
//! (invariant 4): an unknown version is refused rather than reinterpreted.

use crate::error::{Error, Result};
use std::path::Path;
use telividb_core::{Dim, Metric};

/// Identifies the file. `TVFM` — telividb field meta.
const MAGIC: [u8; 4] = *b"TVFM";
/// Highest version this build writes and reads.
const VERSION: u16 = 1;
/// `magic(4) version(2) dim(4) metric(1)`
const BYTES: usize = 11;

/// The immutable facts about one named vector field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldMeta {
    /// Width of every vector in the field.
    pub dim: Dim,
    /// How similarity is measured.
    pub metric: Metric,
}

impl FieldMeta {
    /// Refuse a caller whose expectations disagree with what was stored.
    ///
    /// Reading a field under the wrong dimension would reinterpret its bytes;
    /// under the wrong metric it would rank correctly-read vectors wrongly.
    /// Both are silent, so both are refused here.
    pub fn check(&self, dim: Dim, metric: Metric) -> Result<()> {
        if self.dim != dim {
            return Err(Error::FieldMismatch {
                what: "dimension",
                stored: self.dim.get().to_string(),
                requested: dim.get().to_string(),
            });
        }
        if self.metric != metric {
            return Err(Error::FieldMismatch {
                what: "metric",
                stored: format!("{:?}", self.metric),
                requested: format!("{metric:?}"),
            });
        }
        Ok(())
    }
}

fn path(dir: &Path) -> std::path::PathBuf {
    dir.join("field.meta")
}

/// Read a field's metadata, or `None` if it has never been written.
pub(super) fn read(dir: &Path) -> Result<Option<FieldMeta>> {
    let file = path(dir);
    if !file.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&file)?;
    if bytes.len() != BYTES {
        return Err(Error::Truncated {
            what: "field.meta",
            needed: BYTES,
            found: bytes.len(),
        });
    }
    let magic: [u8; 4] = bytes[0..4].try_into().expect("4 bytes");
    if magic != MAGIC {
        return Err(Error::BadMagic {
            expected: MAGIC,
            found: magic,
        });
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().expect("2 bytes"));
    if version > VERSION {
        return Err(Error::UnsupportedVersion {
            what: "field.meta",
            found: version,
            supported: VERSION,
        });
    }
    Ok(Some(FieldMeta {
        dim: Dim::new(u32::from_le_bytes(
            bytes[6..10].try_into().expect("4 bytes"),
        ))?,
        metric: metric_of(bytes[10])?,
    }))
}

/// Write a field's metadata, once, when the field is created.
pub(super) fn write(dir: &Path, meta: FieldMeta) -> Result<()> {
    let mut out = Vec::with_capacity(BYTES);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(meta.dim.get() as u32).to_le_bytes());
    out.push(metric_byte(meta.metric));
    crate::segment::durable::write_synced(path(dir), &out)
}

fn metric_byte(metric: Metric) -> u8 {
    match metric {
        Metric::Dot => 0,
        Metric::L2 => 1,
        Metric::Cosine => 2,
    }
}

fn metric_of(byte: u8) -> Result<Metric> {
    match byte {
        0 => Ok(Metric::Dot),
        1 => Ok(Metric::L2),
        2 => Ok(Metric::Cosine),
        value => Err(Error::UnknownDiscriminant {
            what: "metric",
            value,
        }),
    }
}

#[cfg(test)]
#[path = "meta_test.rs"]
mod tests;
