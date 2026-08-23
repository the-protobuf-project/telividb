//! Locating external tools.
//!
//! Shared because more than one task needs it, and because the failure message
//! matters: `buf` and `api-linter` are development tools, so a missing one must
//! say so rather than looking like a broken build.

use std::path::PathBuf;

/// Find `binary` on `PATH`, or `None` if it is not installed.
pub fn which(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.is_file())
    })
}
