//! Making a file or directory durable before it is published.
//!
//! # Why a directory sync is not optional
//!
//! `fsync` on a file makes its *contents* durable. It says nothing about the
//! directory entry naming it. A crash can therefore leave a published segment
//! whose files are individually intact but whose entries were never recorded —
//! the rename is durable, and what it renamed is missing.
//!
//! So the order is: write and sync every file, sync the directory holding them,
//! then rename, then sync the parent that now names the segment. Skipping any
//! step publishes a segment that may not be entirely there, which is the one
//! failure the sealed-segment design exists to rule out.

use crate::error::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Write `bytes` to `path` and make the contents durable.
///
/// `fs::write` does not sync. `present.roar` and `codebook.pq` were written
/// with it, so a crash could publish a segment whose presence bitmap or
/// codebook was missing or zero-length — while `raw.bin` beside them was
/// perfectly intact.
pub fn write_synced(path: impl AsRef<Path>, bytes: &[u8]) -> Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

/// Make a directory's entries durable.
///
/// Best-effort on platforms that refuse to open a directory for sync — Windows
/// has no equivalent, and failing a seal there would be worse than the weaker
/// guarantee. On Unix, where the guarantee is real, an error propagates.
pub fn sync_dir(path: impl AsRef<Path>) -> Result<()> {
    let opened = fs::File::open(path.as_ref());
    #[cfg(unix)]
    {
        opened?.sync_all()?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        if let Ok(dir) = opened {
            let _ = dir.sync_all();
        }
        Ok(())
    }
}

/// The temporary directory a segment is built in.
///
/// Appends rather than replaces: `Path::with_extension` would turn both `seg.1`
/// and `seg.2` into `seg.building`, so two seals in the same directory would
/// share a temp path and destroy each other's work.
pub fn building_path(final_path: &Path) -> PathBuf {
    let mut name = final_path.as_os_str().to_os_string();
    name.push(".building");
    PathBuf::from(name)
}
