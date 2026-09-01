//! Provider types. Declarations and re-exports only.

mod guard;
mod provider;

pub use guard::may_answer;
pub use provider::{Locality, PROVIDERS, Provider, provider};
