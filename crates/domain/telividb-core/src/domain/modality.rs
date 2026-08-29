//! What kind of content a model turns into vectors.

use crate::domain::Architecture;

/// The content a model embeds.
///
/// A named vector field binds to one model (rule 12), and a model reads one
/// kind of content. Recording that here is what lets a collection say "this
/// field holds audio" and have the mismatch refused at schema time rather than
/// discovered when a search returns plausible nonsense.
///
/// # Only text works today, and the type says so
///
/// The other variants exist because the ontology needs them and a catalog entry
/// has to be able to name one — not because they are reachable. The encoder in
/// `telividb-embed` implements exactly the architectures in
/// [`Architecture`], which are BERT-family text encoders. Every image, audio
/// and video model published as GGUF carries a different architecture: `clip`
/// for image towers, `whisper` for speech, a generative architecture for the
/// video-capable multimodal models.
///
/// So this is not a switch waiting to be flipped. Reaching audio or video means
/// a new forward pass in layer four, per architecture — and for anything
/// beyond text it also means decoding media, which invariant 19 keeps outside
/// the database entirely. [`Modality::is_supported`] is the honest answer, and
/// callers should route on it rather than assuming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modality {
    /// Natural-language text. The only modality with an encoder behind it.
    Text,
    /// Still images, through a vision tower. No loadable architecture yet.
    Image,
    /// Speech or general audio. No loadable architecture yet.
    Audio,
    /// Video, which in practice means frames plus audio. No loadable
    /// architecture yet, and the furthest from reach of the four.
    Video,
}

impl Modality {
    /// Whether this engine can currently embed this kind of content.
    ///
    /// Derived from [`Architecture`] rather than hard-coded, so it cannot
    /// answer `true` for a modality no architecture serves. Adding a vision
    /// encoder is what makes [`Modality::Image`] supported — not editing this
    /// method.
    pub fn is_supported(&self) -> bool {
        match self {
            Self::Text => !Architecture::NAMES.is_empty(),
            Self::Image | Self::Audio | Self::Video => false,
        }
    }

    /// A lowercase name, as written in a catalog entry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
        }
    }

    /// Parse a catalog entry's `modality` value.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "text" => Some(Self::Text),
            "image" => Some(Self::Image),
            "audio" => Some(Self::Audio),
            "video" => Some(Self::Video),
            _ => None,
        }
    }
}

impl std::fmt::Display for Modality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
#[path = "modality_test.rs"]
mod tests;
