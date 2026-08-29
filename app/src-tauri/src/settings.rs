//! Where the data lives and which port the engine listens on.

use std::net::SocketAddr;
use std::path::PathBuf;
use telividb_desktop_engine::{Engine, Result};

/// Startup settings, resolved from the environment.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Directory holding segments, the write-ahead log and metadata.
    pub data_dir: PathBuf,
    /// Where the engine listens.
    pub addr: SocketAddr,
    /// A GGUF model, when one is configured.
    ///
    /// Without it the server refuses text queries and only precomputed vectors
    /// work. That is a real limit the window states rather than hides.
    pub model: Option<PathBuf>,
}

impl Settings {
    /// Read the environment, falling back to per-user defaults.
    pub fn resolve() -> Self {
        Self {
            data_dir: env_path("TELIVIDB_DATA_DIR").unwrap_or_else(default_data_dir),
            addr: std::env::var("TELIVIDB_ADDR")
                .ok()
                .and_then(|a| a.parse().ok())
                .unwrap_or(DEFAULT_ADDR),
            model: env_path("TELIVIDB_MODEL"),
        }
    }

    /// Start an engine with these settings.
    pub async fn start(&self) -> Result<Engine> {
        Engine::start(self.data_dir.clone(), self.addr, self.model.clone()).await
    }
}

/// A path from the environment, ignoring an empty value.
fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// `~/.telividb/data`, or the nearest per-user directory the platform offers.
///
/// Under the home directory rather than the platform's application-support
/// path, and short on purpose: a Unix socket path is capped at 104 bytes on
/// macOS, and the per-user `/var/folders/...` prefix spends most of that before
/// anything is appended. The socket is not served yet; picking the directory
/// that will still fit when it is costs nothing now.
///
/// Never the working directory. A window launched from Finder or a Start menu
/// inherits `/` — not writable, and not anywhere a person would look for their
/// own data. The last resort is the system temp directory, which is at least
/// writable and obviously wrong, rather than a path that fails at the first
/// write with an error about permissions.
fn default_data_dir() -> PathBuf {
    user_home()
        .unwrap_or_else(std::env::temp_dir)
        .join(".telividb")
        .join("data")
}

/// The user's home, by whichever name this platform gives it.
///
/// `HOME` is set on macOS and Linux. Windows sets `USERPROFILE` instead, and
/// `LOCALAPPDATA` is where per-user application data belongs there — so it is
/// preferred when present.
fn user_home() -> Option<PathBuf> {
    let keys: &[&str] = if cfg!(windows) {
        &["LOCALAPPDATA", "USERPROFILE", "HOME"]
    } else {
        &["HOME"]
    };
    keys.iter()
        .filter_map(|key| std::env::var_os(key))
        .find(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// The loopback port the engine listens on by default.
///
/// Loopback, and only for now. A TCP port on 127.0.0.1 is reachable by every
/// process on the machine, where a Unix socket carries filesystem permissions
/// — which is the transport this is meant to move to.
const DEFAULT_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 7700);
