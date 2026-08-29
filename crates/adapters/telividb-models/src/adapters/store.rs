//! Where installed models live on disk.

use crate::{CatalogEntry, Error, Fetcher, Result};
use std::path::{Path, PathBuf};
use telividb_core::Fingerprint;
use telividb_telemetry::logger;

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
    fn partial_of(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.gguf.part"))
    }

    /// Whether this model is installed and intact.
    ///
    /// Checks the digest rather than merely the path, because the interesting
    /// failure is a file that exists and is wrong — truncated by a full disk,
    /// or replaced.
    ///
    /// Streamed rather than read whole. The catalog calls this once per entry
    /// every time it is listed, and a listing is what the window does on open —
    /// so reading the files into memory would allocate gigabytes to answer a
    /// question about a directory. It still walks every byte, so it is not free;
    /// it is merely bounded.
    pub fn is_installed(&self, entry: &CatalogEntry) -> bool {
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

    /// Download and verify a model, returning where it landed.
    ///
    /// Idempotent: an install that is already present and intact returns
    /// immediately without fetching. An interrupted one resumes from the
    /// partial file rather than starting over.
    ///
    /// `progress` receives cumulative bytes written, including bytes already
    /// on disk from a previous attempt, so a resumed download reports a bar
    /// that continues rather than one that restarts.
    pub fn install(
        &self,
        entry: &CatalogEntry,
        fetcher: &dyn Fetcher,
        progress: &mut dyn FnMut(u64) -> bool,
    ) -> Result<PathBuf> {
        let final_path = self.path_of(&entry.id);
        if self.is_installed(entry) {
            progress(entry.size_bytes);
            return Ok(final_path);
        }

        logger::info!("installing a model").with_data(&serde_json::json!({
            "telividb.model.id": entry.id,
            "telividb.model.architecture": entry.architecture.as_str(),
            "telividb.model.bytes": entry.size_bytes,
        }));

        std::fs::create_dir_all(&self.root)?;
        let partial = self.partial_of(&entry.id);
        let resume_from = std::fs::metadata(&partial).map(|m| m.len()).unwrap_or(0);

        // A partial larger than the expected file is not a resumable download:
        // it is a leftover from a different file under the same id. Start over
        // rather than append to it.
        let resume_from = if resume_from >= entry.size_bytes {
            let _ = std::fs::remove_file(&partial);
            0
        } else {
            resume_from
        };

        let mut sink = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&partial)?;
        fetcher.stream(&entry.download_url(), resume_from, &mut sink, progress)?;
        drop(sink);

        // A cancelled transfer leaves a short file. Verifying it would report a
        // digest mismatch and delete the partial, which is exactly wrong — the
        // bytes are good, there are simply fewer of them than the whole file.
        let written = std::fs::metadata(&partial).map(|m| m.len()).unwrap_or(0);
        if written < entry.size_bytes {
            return Err(Error::Cancelled {
                name: entry.id.clone(),
                written,
            });
        }

        self.promote(entry, &partial, final_path)
    }

    /// Verify a completed download and move it into place.
    ///
    /// The digest is checked before the rename, so the path the engine loads
    /// from never holds unverified bytes — not even briefly. A mismatch
    /// removes the file rather than leaving it to be resumed, because a
    /// resumed download of the wrong file converges on the wrong file.
    fn promote(
        &self,
        entry: &CatalogEntry,
        partial: &Path,
        final_path: PathBuf,
    ) -> Result<PathBuf> {
        let found = Fingerprint::of_reader(std::io::BufReader::new(std::fs::File::open(partial)?))?;
        if found != entry.digest {
            let _ = std::fs::remove_file(partial);
            return Err(Error::DigestMismatch {
                name: entry.id.clone(),
                expected: entry.digest,
                found,
            });
        }
        std::fs::rename(partial, &final_path)?;
        logger::info!("model installed").with_data(&serde_json::json!({
            "telividb.model.id": entry.id,
            "telividb.model.digest": entry.digest.short(),
        }));
        Ok(final_path)
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
