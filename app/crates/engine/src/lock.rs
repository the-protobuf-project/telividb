//! Claiming a data directory before the engine opens it.

use crate::{Error, Result};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

/// An exclusive claim on one data directory, released when dropped.
///
/// `redb` already refuses a second opener, so this adds no safety. What it adds
/// is *when* and *how* the refusal arrives: here, before the engine starts,
/// naming the directory and what to do about it — rather than several layers
/// down as a storage error about a file handle.
///
/// A lock file rather than a pidfile, deliberately. A pidfile outlives the
/// process that wrote it, so a crash leaves a file claiming a process that is
/// gone, and every later start has to guess whether that pid is still the same
/// program. A kernel lock is released when the process dies, however it dies.
#[derive(Debug)]
pub struct DataDirLock {
    /// Held open for as long as the claim lasts. Closing it releases the lock,
    /// so this field is the whole point of the type even though nothing reads
    /// it.
    _file: File,
    /// The directory this claim covers, for the error message.
    path: PathBuf,
}

impl DataDirLock {
    /// Claim `dir`, creating it if it does not exist.
    pub fn acquire(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir).map_err(|source| Error::DataDir {
            path: dir.to_path_buf(),
            source,
        })?;

        let path = dir.join("LOCK");
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .map_err(|source| Error::DataDir {
                path: path.clone(),
                source,
            })?;

        try_lock(&file, dir)?;
        Ok(Self {
            _file: file,
            path: dir.to_path_buf(),
        })
    }

    /// The directory this lock covers.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Take a non-blocking exclusive lock, or report the directory as busy.
///
/// Non-blocking on purpose: a second window should say so immediately rather
/// than hang with no window and no explanation while it waits for a lock the
/// first one will not release.
#[cfg(unix)]
fn try_lock(file: &File, dir: &Path) -> Result<()> {
    use std::os::unix::io::AsRawFd;

    // SAFETY: `flock` takes a file descriptor and a flag, and touches nothing
    // else. The descriptor is valid for the lifetime of `file`, which outlives
    // this call, and the return value is checked below.
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc == 0 {
        return Ok(());
    }

    let err = std::io::Error::last_os_error();
    match err.kind() {
        std::io::ErrorKind::WouldBlock => Err(Error::DataDirBusy(dir.to_path_buf())),
        _ => Err(Error::DataDir {
            path: dir.to_path_buf(),
            source: err,
        }),
    }
}

/// Windows has no `flock`; `redb`'s own exclusive open is the guarantee there.
///
/// The claim is skipped rather than faked. A lock that always succeeds would
/// report a busy directory as free, which is worse than reporting nothing —
/// and `redb` still refuses the second opener, so the safety is unchanged.
#[cfg(not(unix))]
fn try_lock(_file: &File, _dir: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
#[path = "lock_test.rs"]
mod tests;
