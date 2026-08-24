//! A [`BlockReader`] backed by an ordinary file.
//!
//! Deliberately not `mmap` yet. Positional reads are correct everywhere, need
//! no `unsafe`, and make no assumptions about page cache behaviour — which is
//! the right starting point while the segment format is still settling. The
//! mmap adapter slots in behind the same port when there is a benchmark to
//! justify it, and nothing above this line changes.

use crate::error::{Error, Result};
use crate::ports::BlockReader;
use std::fs::File;
use std::path::Path;

/// Positional reads over one segment file.
#[derive(Debug)]
pub struct FileBlockReader {
    file: File,
    len: u64,
}

impl FileBlockReader {
    /// Open a segment file for positional reads.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        Ok(Self { file, len })
    }
}

impl BlockReader for FileBlockReader {
    fn len(&self) -> u64 {
        self.len
    }

    fn read_exact_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        let end = offset.saturating_add(buf.len() as u64);
        if end > self.len {
            return Err(Error::Truncated {
                what: "segment file",
                needed: end as usize,
                found: self.len as usize,
            });
        }
        read_at(&self.file, offset, buf)
    }
}

#[cfg(unix)]
fn read_at(file: &File, offset: u64, buf: &mut [u8]) -> Result<()> {
    use std::os::unix::fs::FileExt;
    file.read_exact_at(buf, offset)?;
    Ok(())
}

#[cfg(not(unix))]
fn read_at(file: &File, offset: u64, buf: &mut [u8]) -> Result<()> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = file;
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(buf)?;
    Ok(())
}

/// A [`BlockReader`] over bytes already in memory. For tests, and for the
/// unsealed buffer's spill path.
#[derive(Debug, Clone)]
pub struct MemoryBlockReader(Vec<u8>);

impl MemoryBlockReader {
    /// Wrap an in-memory buffer as a block reader.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl BlockReader for MemoryBlockReader {
    fn len(&self) -> u64 {
        self.0.len() as u64
    }

    fn read_exact_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        let start = offset as usize;
        let end = start + buf.len();
        let Some(slice) = self.0.get(start..end) else {
            return Err(Error::Truncated {
                what: "memory block",
                needed: end,
                found: self.0.len(),
            });
        };
        buf.copy_from_slice(slice);
        Ok(())
    }
}
