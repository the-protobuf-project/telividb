//! The engine, running inside the desktop process.
//!
//! # Why this is not part of the Tauri crate
//!
//! Nothing here mentions Tauri. Supervising a server — starting it, holding a
//! way to stop it, refusing to start a second one on the same data — is
//! ordinary Rust with ordinary failure modes, and keeping it apart means it can
//! be tested with `cargo test` and no window.
//!
//! # The shape, and why it is this one
//!
//! The app spawns [`telividb_server::serve`] on a background task and talks to
//! it through [`telividb_client`], the same gRPC client any other program would
//! use. That is Ollama's shape: a window supervising a server rather than
//! replacing it.
//!
//! It costs a serialization round trip inside one process, which the
//! architecture calls waste on the distributed path and would call waste here.
//! It buys something worth more for now: the desktop commands cannot
//! accumulate logic, because a shim forwarding to a gRPC client has nowhere to
//! put any. The rule that the app and a browser must reach identical behaviour
//! holds by construction rather than by discipline.
//!
//! Closing that gap later means `serve` handing back service handles instead of
//! consuming them. Callers of this crate would not notice.

mod connect;
mod error;
mod handle;
mod lock;

pub use error::{Error, Result};
pub use handle::Engine;
pub use lock::DataDirLock;
