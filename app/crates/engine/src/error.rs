//! What can go wrong starting or reaching the engine.

use std::path::PathBuf;

/// A failure starting, locking, or connecting to the embedded engine.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Another process already owns this data directory.
    ///
    /// `redb` takes an exclusive file lock, so this is refused rather than
    /// corrupted either way — segments are `mmap`'d and the write-ahead log has
    /// a single writer, so two engines over one directory is corruption, not
    /// contention. What this variant adds is a legible message: without it the
    /// same situation surfaces as a storage error about a file that cannot be
    /// opened, which reads like a bug rather than a second copy running.
    #[error(
        "another telividb is already using {}.\n\
         Quit the other window, or choose a different data directory.",
        .0.display()
    )]
    DataDirBusy(PathBuf),

    /// The data directory could not be created or locked.
    #[error("data directory {path}: {source}")]
    DataDir {
        /// The directory that could not be prepared.
        path: PathBuf,
        /// The underlying filesystem failure.
        source: std::io::Error,
    },

    /// The server stopped before it began accepting connections.
    ///
    /// Carries the server's own message. The common causes are a port already
    /// bound and a telemetry pipeline that failed to install — that one is
    /// fatal by design, because a server whose telemetry silently did not start
    /// is a server nobody can debug later.
    #[error("the engine did not start: {0}")]
    Startup(String),

    /// Something else already holds the port this engine was told to serve on.
    ///
    /// Checked before the server is started, and the reason is a real failure
    /// this replaced: the engine would spawn, fail to bind, and the client would
    /// meanwhile connect to *whoever else* was on the port — reporting nothing,
    /// and answering every later call from a stranger's server. That surfaced
    /// hours later as `Unimplemented` on a service this build definitely has.
    ///
    /// A leftover from an earlier build is the usual cause, so the message says
    /// how to find it rather than only that it happened.
    #[error(
        "{addr} is already in use, so this engine did not start. Something else \
         is serving there — often a previous build still running. Find it with \
         `lsof -nP -iTCP:{port} -sTCP:LISTEN` and stop it, or set TELIVIDB_ADDR \
         to a free port."
    )]
    PortBusy {
        /// The address that could not be claimed.
        addr: std::net::SocketAddr,
        /// Its port, repeated so the suggested command can name it.
        port: u16,
    },

    /// The client could not reach the server that was just started.
    #[error("could not reach the engine: {0}")]
    Connect(#[from] telividb_client::Error),
}

/// A desktop-engine result.
pub type Result<T> = std::result::Result<T, Error>;
