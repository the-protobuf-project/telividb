//! Whether this content may be sent to this provider.

use super::Provider;
use crate::{Error, Result};
use telividb_core::Protection;

/// Refuse a provider that would take protected content off the machine.
///
/// A vault or sealed space may be answered by a model on this machine and by
/// nothing else. A vault whose passages were sent to a third party after someone
/// clicked through a dialog was never a vault, so this refuses rather than warns.
///
/// **Nothing calls this yet, and that is the honest state of the rule.** The
/// window calls the provider SDKs directly, so the check that runs today is the
/// TypeScript one beside that call — and a check inside one client is a check the
/// next client does not have. This function is the server-side half, waiting on
/// the piece that makes it unbypassable: a search that declares whether its
/// passages are bound for a remote model, so the engine can decline to *return*
/// protected passages rather than trusting the caller not to forward them. Until
/// that lands, treat the badge as describing an intention.
///
/// Kept in Rust rather than deleted because it is the rule the retrieval-side
/// check will enforce, and because rule 25 is easier to keep true when the
/// statement of it lives somewhere a client cannot edit.
pub fn may_answer(space: &str, protection: Protection, provider: &Provider) -> Result<()> {
    let protected = matches!(protection, Protection::Vault | Protection::Sealed);
    if !protected || provider.is_local() {
        return Ok(());
    }
    Err(Error::WouldLeaveMachine {
        space: space.to_owned(),
        protection: match protection {
            Protection::Sealed => "sealed",
            _ => "key-wrapped",
        }
        .to_owned(),
        provider: provider.display_name.to_owned(),
    })
}

#[cfg(test)]
#[path = "guard_test.rs"]
mod tests;
