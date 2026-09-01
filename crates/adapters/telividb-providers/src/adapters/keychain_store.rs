//! A [`SecretStore`] backed by the operating system's own.

use crate::{Error, Result, SecretStore};

/// Keeps credentials where the platform keeps them.
///
/// macOS Keychain, Windows Credential Manager, or Secret Service on Linux —
/// chosen by `keyring` at compile time for the target, not by anything here.
///
/// The engine holds these **at rest**, which is what this buys and the limit of
/// what it buys. A key is written to the platform store rather than to a file the
/// app can read, so it survives an uninstall correctly, is protected by the login
/// keychain, and is never in the app's own configuration.
///
/// It is not out of the window's reach. Answering happens in the window — the
/// provider SDKs are TypeScript — so the credential is handed over IPC for the
/// duration of a call and is in webview memory while it runs. The exposure that
/// matters is script injected through a rendered passage, and nothing here
/// prevents it. Do not describe these keys as unreachable from the frontend.
pub struct KeychainStore {
    /// Namespaces entries so this app's keys are its own.
    service: String,
}

impl KeychainStore {
    /// A store writing under `service`.
    ///
    /// The service name is what a person will see in Keychain Access, so it is
    /// the product's name rather than a crate's.
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    /// The platform entry for one provider.
    fn entry(&self, provider: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(&self.service, provider).map_err(|e| Error::Store(e.to_string()))
    }
}

impl SecretStore for KeychainStore {
    fn get(&self, provider: &str) -> Result<Option<String>> {
        match self.entry(provider)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            // Absent is an answer, not a failure — it is the state of every
            // provider before one is configured.
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::Store(e.to_string())),
        }
    }

    fn set(&self, provider: &str, secret: &str) -> Result<()> {
        self.entry(provider)?
            .set_password(secret)
            .map_err(|e| Error::Store(e.to_string()))
    }

    fn clear(&self, provider: &str) -> Result<()> {
        match self.entry(provider)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::Store(e.to_string())),
        }
    }
}
