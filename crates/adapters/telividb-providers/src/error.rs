//! What can go wrong reaching a provider.

/// A provider could not be configured, reached, or trusted with the request.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No provider is registered under that name.
    #[error("no provider called {0:?}")]
    Unknown(
        /// The name that was asked for.
        String,
    ),

    /// The provider needs a key and none is stored.
    #[error("{0} needs an API key before it can answer; add one in settings")]
    NoKey(
        /// The provider that is not configured.
        String,
    ),

    /// The content may not leave this machine.
    ///
    /// Not a transport failure and not a permission check — a statement about
    /// what the space *is*. A vault whose contents are sent to a third party
    /// was never a vault, so this is refused rather than warned about.
    #[error(
        "{space} is {protection}, so its contents are answered by a model on this \
         machine rather than a remote one. {provider} is remote — choose a local \
         provider, or ask in a private space."
    )]
    WouldLeaveMachine {
        /// The space that was being read.
        space: String,
        /// Its protection, named as the person sees it.
        protection: String,
        /// The provider that was refused.
        provider: String,
    },

    /// The keychain refused, or holds nothing under that name.
    #[error("the key store: {0}")]
    Store(String),
}

/// The result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
