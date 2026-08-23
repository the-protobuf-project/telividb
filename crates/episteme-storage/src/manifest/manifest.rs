//! Which segments make up a collection, right now.

use crate::error::{Error, Result};
use episteme_telemetry::{fields, metrics_names};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

pub const MANIFEST_VERSION: u16 = 1;
const MANIFEST_MAGIC: [u8; 4] = *b"EPMF";

/// The set of segments visible at one generation.
///
/// This is the only thing in a collection that changes, and it changes by
/// atomic rename. Everything it points at is immutable, so publishing a batch
/// of new segments is one indivisible step: readers see the old set or the new
/// set, never a mixture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Manifest {
    /// Monotonic; increments on every successful write.
    pub generation: u64,
    /// Segment ids, in creation order.
    pub segments: Vec<u64>,
}

impl Manifest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Produce the next generation with `segment` added.
    ///
    /// Takes `self` by value and returns a new manifest rather than mutating in
    /// place — publishing is a swap of whole values, and modelling it that way
    /// keeps a partially-updated manifest unrepresentable.
    pub fn with_segment(mut self, segment: u64) -> Self {
        self.segments.push(segment);
        self.generation += 1;
        self
    }

    /// Produce the next generation with `removed` dropped — how compaction
    /// retires the inputs it merged.
    pub fn without_segments(mut self, removed: &[u64]) -> Self {
        self.segments.retain(|id| !removed.contains(id));
        self.generation += 1;
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(18 + self.segments.len() * 8);
        out.extend_from_slice(&MANIFEST_MAGIC);
        out.extend_from_slice(&MANIFEST_VERSION.to_le_bytes());
        out.extend_from_slice(&self.generation.to_le_bytes());
        out.extend_from_slice(&(self.segments.len() as u32).to_le_bytes());
        for id in &self.segments {
            out.extend_from_slice(&id.to_le_bytes());
        }
        let crc = crc32fast::hash(&out);
        out.extend_from_slice(&crc.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        const PREFIX: usize = 18;
        if bytes.len() < PREFIX + 4 {
            return Err(Error::Truncated {
                what: "manifest",
                needed: PREFIX + 4,
                found: bytes.len(),
            });
        }

        let found: [u8; 4] = bytes[0..4].try_into().expect("4 bytes");
        if found != MANIFEST_MAGIC {
            return Err(Error::BadMagic {
                expected: MANIFEST_MAGIC,
                found,
            });
        }

        let version = u16::from_le_bytes(bytes[4..6].try_into().expect("2 bytes"));
        if version > MANIFEST_VERSION {
            return Err(Error::UnsupportedVersion {
                what: "manifest",
                found: version,
                supported: MANIFEST_VERSION,
            });
        }

        let body = bytes.len() - 4;
        let expected = u32::from_le_bytes(bytes[body..].try_into().expect("4 bytes"));
        let computed = crc32fast::hash(&bytes[..body]);
        if expected != computed {
            return Err(Error::Corrupt {
                what: "manifest",
                expected,
                computed,
            });
        }

        let count = u32::from_le_bytes(bytes[14..18].try_into().expect("4 bytes")) as usize;
        let needed = PREFIX + count * 8 + 4;
        if bytes.len() != needed {
            return Err(Error::Truncated {
                what: "manifest segment list",
                needed,
                found: bytes.len(),
            });
        }

        let segments = bytes[PREFIX..body]
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().expect("8 bytes")))
            .collect();

        Ok(Self {
            generation: u64::from_le_bytes(bytes[6..14].try_into().expect("8 bytes")),
            segments,
        })
    }

    /// Publish atomically: write a sibling temp file, fsync it, then rename
    /// over `path`.
    ///
    /// On POSIX the rename is atomic, so a reader sees the whole previous
    /// manifest or the whole new one. A crash at any point leaves one of the
    /// two intact — never a half-written pointer to segments that may not exist.
    pub fn write_atomic(&self, path: impl AsRef<Path>) -> Result<()> {
        let span = tracing::info_span!(
            "episteme.manifest.swap",
            { fields::GENERATION } = self.generation,
            segments = self.segments.len(),
        );
        let _guard = span.enter();
        let started = Instant::now();

        let path = path.as_ref();
        let tmp = path.with_extension("tmp");

        let mut file = fs::File::create(&tmp)?;
        file.write_all(&self.encode())?;
        file.sync_all()?;
        drop(file);

        fs::rename(&tmp, path)?;

        // Fsync the directory too, or the rename itself may not survive a
        // power loss even though the file contents did.
        if let Some(dir) = path.parent() {
            let _ = fs::File::open(dir).and_then(|d| d.sync_all());
        }

        metrics::histogram!(metrics_names::MANIFEST_SWAP_DURATION)
            .record(started.elapsed().as_secs_f64());
        metrics::gauge!(metrics_names::SEGMENTS_LIVE).set(self.segments.len() as f64);
        tracing::debug!("manifest published");
        Ok(())
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        Self::decode(&fs::read(path)?)
    }
}

#[cfg(test)]
#[path = "manifest_test.rs"]
mod tests;
