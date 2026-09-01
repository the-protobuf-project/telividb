//! Remembering that a file was already verified.
//!
//! Verifying a model means reading it whole and hashing it — about 1.5 seconds
//! for a 639 MB file, and the store did that on *every* startup before handing
//! the file to the loader. Worse when several models are installed: the fallback
//! walks the catalog and hashes each installed entry until one verifies.
//!
//! That is a real cost paid forever to catch a rare event. This records the
//! result so it is paid once.
//!
//! # What the receipt is allowed to promise
//!
//! Only that *this exact file* was verified. It stores the size and the
//! modification time alongside the digest, and a mismatch in either sends the
//! caller back to a full hash. So a replaced file, a truncated file, or one
//! rewritten in place are all caught — the cases that actually happen.
//!
//! It does not catch silent bit-rot that leaves size and mtime intact. Nothing
//! short of re-reading does, and re-reading is what this exists to avoid. The
//! trade is stated rather than hidden: a corrupt-but-plausible file fails in the
//! GGUF reader instead of here, which is a worse message for a rarer fault.

use std::path::Path;
use telividb_core::Fingerprint;

/// What a receipt records: the file it describes, and what it hashed to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Receipt {
    /// Size in bytes when it was verified.
    pub size: u64,
    /// Modification time, as seconds since the epoch.
    pub modified: u64,
    /// What the file hashed to.
    pub digest: Fingerprint,
}

impl Receipt {
    /// Where a model's receipt lives: beside the file, not inside it.
    ///
    /// A sibling rather than a sidecar directory, so deleting the model by hand
    /// leaves an orphan next to nothing rather than a stale entry in a registry
    /// that outlives it.
    pub(super) fn path_for(model: &Path) -> std::path::PathBuf {
        model.with_extension("verified")
    }

    /// Read a receipt, if one is there and parses.
    ///
    /// Any malformed line is treated as absent. A receipt is a cache, and a
    /// cache that errors is worse than one that misses.
    pub(super) fn read(model: &Path) -> Option<Self> {
        let raw = std::fs::read_to_string(Self::path_for(model)).ok()?;
        let mut parts = raw.trim().split(':');
        let size = parts.next()?.parse().ok()?;
        let modified = parts.next()?.parse().ok()?;
        let digest = Fingerprint::from_hex(parts.next()?)?;
        Some(Self {
            size,
            modified,
            digest,
        })
    }

    /// Describe the file as it is on disk right now.
    pub(super) fn of(model: &Path, digest: Fingerprint) -> Option<Self> {
        let meta = std::fs::metadata(model).ok()?;
        Some(Self {
            size: meta.len(),
            modified: modified_secs(&meta)?,
            digest,
        })
    }

    /// Whether this receipt still describes the file it was written for.
    pub(super) fn still_describes(&self, model: &Path) -> bool {
        std::fs::metadata(model).ok().is_some_and(|meta| {
            meta.len() == self.size && modified_secs(&meta) == Some(self.modified)
        })
    }

    /// Write it beside the model. A failure here is not worth failing a load
    /// over — the next run simply pays the hash again.
    pub(super) fn write(&self, model: &Path) {
        let line = format!("{}:{}:{}", self.size, self.modified, self.digest.to_hex());
        let _ = std::fs::write(Self::path_for(model), line);
    }
}

/// Modification time in whole seconds, or `None` if the platform withholds it.
fn modified_secs(meta: &std::fs::Metadata) -> Option<u64> {
    meta.modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}
