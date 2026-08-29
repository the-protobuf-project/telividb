//! The model architectures this engine can actually load.

/// A GGUF architecture the encoder implements a forward pass for.
///
/// This is a closed set, and deliberately so. A GGUF file names its
/// architecture in `general.architecture`, and the encoder builds its forward
/// pass from that name — which tensors to expect, how attention is wired, where
/// the norms sit. An unrecognised architecture finds *some* of the tensors it
/// expects and misreads the rest, producing vectors that are plausible and
/// wrong rather than an error.
///
/// It lives here, in the ontology, rather than in the crate that loads models,
/// because two things need it and they must not disagree: the loader, which
/// refuses a file it cannot read, and the catalog, which refuses to *download*
/// one. A second copy of this list is a copy that eventually drifts, and the
/// symptom would be a gigabyte fetched before the refusal.
///
/// Adding a variant is real work in `telividb-embed`, not an entry here. The
/// name buys nothing without a forward pass behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Architecture {
    /// Classic BERT encoders: bge, gte, e5, MiniLM, mxbai and their relatives.
    Bert,
    /// Nomic's BERT variant, which extends the context window past 512 tokens.
    NomicBert,
}

impl Architecture {
    /// Every architecture the encoder implements, by GGUF name.
    ///
    /// The order is stable so error messages read the same way twice.
    pub const NAMES: &'static [&'static str] = &["bert", "nomic-bert"];

    /// Recognise an architecture by the name a GGUF header carries.
    ///
    /// Returns `None` for anything unimplemented, which is the common case:
    /// most GGUF files on any model host are generative models, and several
    /// *embedding* models — EmbeddingGemma's `gemma-embedding`, Qwen3-Embedding's
    /// `qwen3`, E5-Mistral's `llama` — are encoders this loader still cannot
    /// read. A caller turns the `None` into a message naming what it found.
    pub fn from_gguf(name: &str) -> Option<Self> {
        match name {
            "bert" => Some(Self::Bert),
            "nomic-bert" => Some(Self::NomicBert),
            _ => None,
        }
    }

    /// The GGUF name for this architecture.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bert => "bert",
            Self::NomicBert => "nomic-bert",
        }
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
#[path = "architecture_test.rs"]
mod tests;
