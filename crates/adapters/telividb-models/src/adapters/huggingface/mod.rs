//! Resolving a model host's repositories into something installable.
//!
//! The catalog covers the common case; this covers the rest. A person who
//! wants a model that is not listed pastes a repository or a link, and gets the
//! same guarantees: the architecture is checked before the download, and the
//! digest is checked after.
//!
//! What this does *not* do is search. A host carries tens of thousands of GGUF
//! files and nearly all are generative models the encoder cannot read, so a
//! search box would mostly return things that fail — see [`Catalog`](crate::Catalog).

mod reference;

use crate::{Error, Result};
use telividb_core::Fingerprint;

pub use reference::Reference;

/// A GGUF file in a repository, as the host describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFile {
    /// The repository this file came from, as `owner/name`.
    ///
    /// Kept beside the filename so the download URL is rebuilt from what was
    /// resolved, rather than stored — which is what stops a listing from one
    /// repository being used to fetch from another.
    pub repository: String,
    /// The filename within the repository.
    pub file: String,
    /// SHA-256, taken from the host's own record rather than computed.
    ///
    /// Trustworthy enough to *plan* with — it is what the download is verified
    /// against — but the verification still happens locally over the bytes
    /// that arrive, because a digest supplied by the same party serving the
    /// file proves only that the transfer was intact.
    pub digest: Fingerprint,
    /// Exact size in bytes, for a progress indicator that ends where it should.
    pub size_bytes: u64,
}

impl RemoteFile {
    /// Where this file is downloaded from.
    pub fn download_url(&self) -> String {
        format!(
            "https://huggingface.co/{}/resolve/main/{}?download=true",
            self.repository, self.file
        )
    }
}

/// The API endpoint listing a repository's files.
fn tree_url(repo: &str) -> String {
    format!("https://huggingface.co/api/models/{repo}/tree/main")
}

/// Every GGUF file in a repository, largest last.
///
/// Ordered by size because that is the axis a person chooses along: within one
/// repository the files are the same weights at different quantizations, so
/// bigger is more faithful and slower to fetch.
pub fn list_gguf(repo: &str, fetcher: &dyn crate::Fetcher) -> Result<Vec<RemoteFile>> {
    let url = tree_url(repo);
    let body = fetcher.text(&url)?;
    let listing: serde_json::Value = serde_json::from_str(&body).map_err(|e| Error::Fetch {
        url: url.clone(),
        reason: format!("the host's answer was not JSON: {e}"),
    })?;

    let entries = listing.as_array().ok_or_else(|| Error::Fetch {
        url,
        reason: "expected a list of files; the repository may be private or misspelled".to_owned(),
    })?;

    let mut files: Vec<RemoteFile> = entries
        .iter()
        .filter_map(|e| remote_file(repo, e))
        .collect();
    files.sort_by_key(|f| f.size_bytes);
    Ok(files)
}

/// Read one listing entry, keeping only usable GGUF files.
///
/// Entries without LFS metadata are skipped rather than guessed at: a GGUF
/// small enough to be stored inline is not a real model, and one whose digest
/// the host does not publish cannot be verified — which is the guarantee that
/// makes downloading from a pasted repository acceptable at all.
fn remote_file(repo: &str, entry: &serde_json::Value) -> Option<RemoteFile> {
    let path = entry.get("path")?.as_str()?;
    if !path.ends_with(".gguf") {
        return None;
    }
    let lfs = entry.get("lfs")?;
    // The listing endpoint calls it `oid`; the model-info endpoint calls the
    // same value `sha256`. Both are the file's SHA-256.
    let digest = lfs
        .get("oid")
        .or_else(|| lfs.get("sha256"))?
        .as_str()
        .and_then(Fingerprint::from_hex)?;
    Some(RemoteFile {
        repository: repo.to_owned(),
        file: path.to_owned(),
        digest,
        size_bytes: lfs.get("size").and_then(serde_json::Value::as_u64)?,
    })
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
