//! A [`SecretStore`] that keeps nothing.

use crate::{Result, SecretStore};
use std::collections::HashMap;
use std::sync::RwLock;

/// Holds credentials for the life of the process and no longer.
///
/// Not only a test double. It is the right store where there is no keychain to
/// reach — a container, a CI run, a headless box with no session bus — and it
/// is honest about what it gives up: keys are supplied per run, and nothing is
/// written where it could be read later.
#[derive(Default)]
pub struct MemoryStore {
    /// Provider id to credential.
    entries: RwLock<HashMap<String, String>>,
}

impl MemoryStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MemoryStore {
    fn get(&self, provider: &str) -> Result<Option<String>> {
        Ok(self
            .entries
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(provider)
            .cloned())
    }

    fn set(&self, provider: &str, secret: &str) -> Result<()> {
        self.entries
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(provider.to_owned(), secret.to_owned());
        Ok(())
    }

    fn clear(&self, provider: &str) -> Result<()> {
        self.entries
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(provider);
        Ok(())
    }
}

#[cfg(test)]
#[path = "memory_store_test.rs"]
mod tests;
