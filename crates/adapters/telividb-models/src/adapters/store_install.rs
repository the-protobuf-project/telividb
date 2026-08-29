//! Fetching a model into the store.
//!
//! Split from `store.rs` because that file is about the directory — where a
//! model lives, whether it is there, what is installed — and this is about
//! getting one. The rules that matter here are all about *not* trusting what
//! arrives: resume only from a partial that could be this file, verify before
//! promoting, and never leave unverified bytes where the engine loads from.

use super::ModelStore;
use crate::{CatalogEntry, Error, Fetcher, Result};
use std::path::{Path, PathBuf};
use telividb_core::Fingerprint;
use telividb_telemetry::logger;

impl ModelStore {
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
        if self.is_verified(entry) {
            progress(entry.size_bytes);
            return Ok(final_path);
        }

        logger::info!("installing a model").with_data(&serde_json::json!({
            "telividb.model.id": entry.id,
            "telividb.model.architecture": entry.architecture.as_str(),
            "telividb.model.bytes": entry.size_bytes,
        }));

        std::fs::create_dir_all(self.root())?;
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
}
