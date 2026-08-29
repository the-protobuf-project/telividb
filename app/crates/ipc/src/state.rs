//! What the commands are allowed to hold.

use std::net::SocketAddr;
use telividb_client::Client;
use telividb_desktop_engine::Engine;

/// Everything a command may reach.
///
/// A client and an address. Deliberately nothing else: a command with no state
/// to consult cannot grow a decision, which is how the rule that logic stays
/// out of this layer is kept by construction rather than by review.
///
/// The [`Engine`] is held so the server stops when the app does — dropping it
/// drops the shutdown sender, and `serve` treats that as a stop.
pub struct AppState {
    /// Owns the running server and the data-directory claim.
    engine: Engine,
}

impl AppState {
    /// Wrap a started engine.
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }

    /// A client reaching the engine. Cheap to clone; the channel multiplexes.
    pub fn client(&self) -> Client {
        self.engine.client()
    }

    /// Where the engine is listening.
    pub fn addr(&self) -> SocketAddr {
        self.engine.addr()
    }
}
