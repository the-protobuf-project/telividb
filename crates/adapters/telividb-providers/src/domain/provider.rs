//! Which models can answer, and where they run.

/// Where a provider runs, which decides what may be sent to it.
///
/// The distinction carries a guarantee rather than a preference: a key-wrapped
/// space may be answered by a model on this machine and by nothing else, so
/// locality is checked before a request is built rather than mentioned in a
/// warning afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locality {
    /// Runs on this machine. Nothing leaves it.
    Local,
    /// Reached over the network. The prompt and its passages leave the machine.
    Remote,
}

/// A model host this build can talk to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    /// Stable id, used in configuration and on the wire.
    pub id: &'static str,
    /// Name shown to a person.
    pub display_name: &'static str,
    /// Whether the prompt leaves the machine.
    pub locality: Locality,
    /// What it is, in the terms that decide whether to use it.
    pub note: &'static str,
    /// Models it offers, most useful first.
    pub models: &'static [&'static str],
    /// What its credential looks like, shown as placeholder text.
    pub credential_hint: &'static str,
}

impl Provider {
    /// Whether the prompt stays on this machine.
    pub fn is_local(&self) -> bool {
        self.locality == Locality::Local
    }

    /// Whether it needs a stored credential before it can be used.
    ///
    /// A local provider is reached by address rather than by key, so it is
    /// usable the moment it is running.
    pub fn needs_key(&self) -> bool {
        !self.is_local()
    }
}

/// Every provider this build knows.
///
/// A fixed list rather than something discovered: each entry is a request shape
/// that had to be written, so one that is not here is not merely unconfigured —
/// it is unimplemented.
pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "ollama",
        display_name: "Ollama",
        locality: Locality::Local,
        note: "Runs on this machine. Nothing leaves it, and it is the only kind \
               of provider a vault will use.",
        models: &["llama3.2", "qwen2.5", "mistral", "phi4"],
        credential_hint: "http://localhost:11434",
    },
    Provider {
        id: "openai",
        display_name: "OpenAI",
        locality: Locality::Remote,
        note: "Sends the question and the retrieved passages.",
        models: &["gpt-4o", "gpt-4o-mini"],
        credential_hint: "sk-…",
    },
    Provider {
        id: "anthropic",
        display_name: "Anthropic",
        locality: Locality::Remote,
        note: "Sends the question and the retrieved passages.",
        models: &["claude-sonnet-4-5", "claude-opus-4-5", "claude-haiku-4-5"],
        credential_hint: "sk-ant-…",
    },
    Provider {
        id: "gemini",
        display_name: "Gemini",
        locality: Locality::Remote,
        note: "Sends the question and the retrieved passages.",
        models: &["gemini-2.0-flash", "gemini-2.0-pro"],
        credential_hint: "AIza…",
    },
    Provider {
        id: "openrouter",
        display_name: "OpenRouter",
        locality: Locality::Remote,
        note: "One key reaches many models, including ones with no adapter here.",
        models: &[
            "openai/gpt-4o",
            "anthropic/claude-sonnet-4.5",
            "meta-llama/llama-3.3-70b-instruct",
        ],
        credential_hint: "sk-or-…",
    },
];

/// Look one up by id.
pub fn provider(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.id == id)
}

#[cfg(test)]
#[path = "provider_test.rs"]
mod tests;
