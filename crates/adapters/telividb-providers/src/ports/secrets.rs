//! Where a provider's credential is kept.

use crate::Result;

/// Stores one secret per provider.
///
/// A port because the right store differs by where the engine runs: a desktop
/// has an OS keychain, a headless daemon may have only the kernel keyring or a
/// file with restricted permissions, and a test wants neither.
///
/// **There is no listing, and that is the only guarantee this port makes.** A
/// caller can ask whether a provider is configured without pulling the value, and
/// nothing enumerates what is stored — so a compromised caller cannot sweep the
/// store. It cannot claim more than that: [`get`](Self::get) exists, the window
/// calls it to make a request, and a key therefore does reach the window. See
/// `keychain_store` for what that costs.
pub trait SecretStore: Send + Sync {
    /// Read the credential for `provider`, if one is stored.
    fn get(&self, provider: &str) -> Result<Option<String>>;

    /// Store a credential, replacing any already there.
    fn set(&self, provider: &str, secret: &str) -> Result<()>;

    /// Forget a credential. Forgetting one that is absent is not an error —
    /// the caller wanted it gone, and it is.
    fn clear(&self, provider: &str) -> Result<()>;

    /// Whether a credential is stored, without reading it.
    ///
    /// Separate from [`get`](Self::get) so the common question — should this
    /// provider be offered? — never has to pull a secret into memory.
    fn has(&self, provider: &str) -> bool {
        self.get(provider).ok().flatten().is_some()
    }
}
