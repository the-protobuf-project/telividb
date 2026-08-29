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
    /// Whether the engine was started with an embedding model.
    ///
    /// Reported rather than inferred. Without one the server refuses text —
    /// for storage as well as for search — and a window that only discovered
    /// that after a person had chosen a file and mapped its columns would have
    /// wasted the work it asked for.
    ///
    /// This is the app's own configuration, not a decision about the engine:
    /// the app is what passes `--model`, so it is the only thing that knows
    /// before asking.
    has_model: bool,
}

impl AppState {
    /// Wrap a started engine, recording whether a model came with it.
    pub fn new(engine: Engine, has_model: bool) -> Self {
        Self { engine, has_model }
    }

    /// Whether text can be embedded, for storage or for search.
    pub fn has_model(&self) -> bool {
        self.has_model
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
