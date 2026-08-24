//! Finding a downloaded model.

use std::path::{Path, PathBuf};

/// Modality a model works on, which is the directory it lives in.
///
/// A closed enum rather than a string: an example asks for "a text embedder"
/// and gets whichever one is present, so adding an image model later is a new
/// variant and a new directory — not a rename of everything already here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Text embedding models.
    Text,
    /// Image and joint image/text models.
    Image,
    /// Audio models — transcription and speaker embedding.
    Audio,
}

impl Category {
    /// The directory name under `examples/models/gguf/`.
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Text => "text",
            Category::Image => "image",
            Category::Audio => "audio",
        }
    }
}

/// Where `examples/models/download.sh` puts its files.
///
/// Resolved from this file's own location rather than the working directory,
/// so `cargo run` works from anywhere in the workspace — a relative path only
/// works from the repository root, which is a papercut every reader hits once.
pub fn models_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("models"))
        .unwrap_or_else(|| PathBuf::from("models"))
}

/// Any verified model in `category`, preferring `preferred` if it is present.
///
/// Returns `Err` with the exact command to run rather than panicking: a
/// missing 80 MiB download is the most likely way a reader's first run fails,
/// and a backtrace would say nothing useful about it.
///
/// Any quantization will do — a reader who fetched `Q8_0` should not be told
/// to fetch `Q4_K_M` as well — so this takes whichever `.gguf` is there.
pub fn find(category: Category, preferred: &str) -> Result<PathBuf, String> {
    let dir = models_dir().join("gguf").join(category.as_str());

    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "gguf"))
        .collect();
    // Sorted so a directory holding several models picks the same one every
    // run; an example whose output changes between runs is hard to trust.
    candidates.sort();

    if let Some(exact) = candidates.iter().find(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with(preferred))
    }) {
        return Ok(exact.clone());
    }

    candidates.into_iter().next().ok_or_else(|| {
        format!(
            "no {} GGUF model found in {}.\n\nFetch one with:\n    {}/download.sh {}",
            category.as_str(),
            dir.display(),
            models_dir().display(),
            category.as_str(),
        )
    })
}

/// The text embedder the walkthrough expects.
pub fn default_text_model() -> Result<PathBuf, String> {
    find(Category::Text, "nomic-embed-text-v1.5")
}
