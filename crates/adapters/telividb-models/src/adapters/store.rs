//! Where installed models live on disk.

use crate::{Catalog, CatalogEntry, Result};
use std::path::{Path, PathBuf};
use telividb_core::Fingerprint;

/// The directory holding downloaded model files.
///
/// One file per catalog id, named for it, in a directory a person can open and
/// understand. Not content-addressed: a store of `sha256-a3f9....gguf` is
/// tidier for deduplication that never happens here — the catalog has no two
/// entries sharing a file — and it makes the directory unreadable to whoever
/// has to check what is installed.
#[derive(Debug, Clone)]
pub struct ModelStore {
    /// The directory. Created on first install rather than at construction, so
    /// listing an install that never happened does not leave a directory behind.
    root: PathBuf,
}

impl ModelStore {
    /// A store rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The directory holding model files.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Where a given model's file belongs.
    pub fn path_of(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.gguf"))
    }

    /// The partial file an interrupted download leaves behind.
    ///
    /// A distinct name, so a half-downloaded file is never mistaken for a
    /// finished one — the engine loads what [`path_of`](Self::path_of) names,
    /// and a truncated GGUF there would fail at load with a confusing error
    /// rather than resume.
    pub(super) fn partial_of(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.gguf.part"))
    }

    /// Whether this model's file is present at its full length.
    ///
    /// **Cheap on purpose — one `stat`, no reading.** This is what a listing
    /// asks, once per entry, every time the catalog is shown. Hashing here
    /// instead meant a 639 MB file was read and digested every time the window
    /// opened the models page, which is exactly the delay that made the page
    /// feel broken.
    ///
    /// Length rather than mere existence, because the failure that actually
    /// happens is a partial file: an interrupted download leaves a `.part`, but
    /// a disk that filled mid-rename can leave a short one under the real name.
    /// A file of the right length that is nevertheless the wrong bytes is not
    /// caught here — [`is_verified`](Self::is_verified) is, and it runs before
    /// the model is ever loaded.
    pub fn is_installed(&self, entry: &CatalogEntry) -> bool {
        std::fs::metadata(self.path_of(&entry.id))
            .is_ok_and(|meta| meta.is_file() && meta.len() == entry.size_bytes)
    }

    /// Whether the file is byte-for-byte the one the catalog names.
    ///
    /// Reads the whole file, so it is for the moments that justify it: before
    /// loading a model, and after a download. Never for a listing.
    pub fn is_verified(&self, entry: &CatalogEntry) -> bool {
        self.digest_of(&self.path_of(&entry.id))
            .is_some_and(|found| found == entry.digest)
    }

    /// The digest of a file, or `None` if it cannot be read.
    ///
    /// Absence and failure are deliberately the same answer here: both mean
    /// "not usable", and the callers cannot act differently on the difference.
    fn digest_of(&self, path: &Path) -> Option<Fingerprint> {
        let file = std::fs::File::open(path).ok()?;
        Fingerprint::of_reader(std::io::BufReader::new(file)).ok()
    }

    /// An installed model to load, and where its file is.
    ///
    /// Prefers the catalog's recommendation, then falls back to whichever entry
    /// is installed — so a first run that installed the default gets it, and a
    /// run that installed something else still gets a model rather than none.
    ///
    /// Verifies the digest, because loading a truncated file fails deep in the
    /// GGUF reader with a message about tensors rather than about the download.
    pub fn resident_choice<'c>(&self, catalog: &'c Catalog) -> Option<&'c CatalogEntry> {
        // Verified, not merely present. This is the one caller that is about to
        // hand the file to the loader, and a truncated GGUF fails deep inside
        // it with a message about tensors rather than about the download.
        let verified = |e: &&'c CatalogEntry| self.is_verified(e);
        catalog
            .recommended()
            .filter(|e| self.is_verified(e))
            .or_else(|| catalog.entries().iter().find(verified))
    }

    /// The ids currently installed, in no particular order.
    ///
    /// By filename rather than by digest, so this stays cheap; use
    /// [`is_installed`](Self::is_installed) when correctness matters.
    pub fn installed_ids(&self) -> Result<Vec<String>> {
        // A missing directory means nothing is installed, which is the normal
        // state of a fresh data directory. Anything else — a permissions
        // failure, a file where the directory should be — is reported rather
        // than flattened into "nothing", because that would present as an empty
        // catalog and offer downloads that then fail for the same reason.
        let entries = match std::fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        let mut ids = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(id) = name.strip_suffix(".gguf") {
                ids.push(id.to_owned());
            }
        }
        ids.sort();
        Ok(ids)
    }
}

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;
