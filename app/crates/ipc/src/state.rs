//! What the commands are allowed to hold.

use std::net::SocketAddr;
use std::sync::Mutex;
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
    ///
    /// Takeable, because quitting has to be able to *consume* it.
    /// `Engine::shutdown` takes `self` — it has to, since it awaits the server
    /// task and only then releases the directory claim — while Tauri hands
    /// managed state out by shared reference. Without somewhere to move the
    /// engine out to, the app could only ever drop it, and on macOS it does not
    /// even get to do that: `NSApplication terminate:` calls `exit()` directly,
    /// so nothing owned by this struct is dropped at all.
    ///
    /// That is not a tidiness point. `exit()` runs ggml's static destructors,
    /// which free the Metal device, which asserts every residency set was
    /// released first — so a quit that skips this teardown aborts with SIGABRT
    /// inside `ggml_metal_rsets_free`.
    engine: Mutex<Option<Engine>>,
    /// Where the data directory is, for the window to show and an operator to
    /// find. The app chose it, so the app is what can report it.
    data_dir: String,
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
    /// Wrap a started engine, recording what the app configured it with.
    pub fn new(engine: Engine, has_model: bool, data_dir: String) -> Self {
        Self {
            engine: Mutex::new(Some(engine)),
            data_dir,
            has_model,
        }
    }

    /// The directory this engine owns.
    pub fn data_dir(&self) -> &str {
        &self.data_dir
    }

    /// Whether text can be embedded, for storage or for search.
    pub fn has_model(&self) -> bool {
        self.has_model
    }

    /// A client reaching the engine. Cheap to clone; the channel multiplexes.
    ///
    /// # Panics
    ///
    /// After [`AppState::take_engine`] has run, which happens once, while the
    /// application is quitting. A command arriving after that is a command
    /// racing process exit, and there is no useful answer to give it.
    pub fn client(&self) -> Client {
        self.with_engine(Engine::client)
    }

    /// Where the engine is listening.
    ///
    /// # Panics
    ///
    /// As [`AppState::client`] does, and for the same reason.
    pub fn addr(&self) -> SocketAddr {
        self.with_engine(Engine::addr)
    }

    /// Take the engine so it can be shut down.
    ///
    /// Returns `None` on a second call, so a quit path that fires twice — the
    /// last window closing and then the application terminating — shuts down
    /// once rather than panicking on the second.
    pub fn take_engine(&self) -> Option<Engine> {
        self.engine
            .lock()
            .expect("the engine lock is only held for a field read")
            .take()
    }

    /// Read something off the engine while it is still there.
    fn with_engine<T>(&self, read: impl FnOnce(&Engine) -> T) -> T {
        let engine = self
            .engine
            .lock()
            .expect("the engine lock is only held for a field read");
        read(engine
            .as_ref()
            .expect("the engine is present until the application quits"))
    }
}
