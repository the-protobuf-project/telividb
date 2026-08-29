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

/// `~/.telividb/data`, or the working directory if there is no home.
///
/// Under the home directory rather than the platform's application-support
/// path, and short on purpose: a Unix socket path is capped at 104 bytes on
/// macOS, and the per-user `/var/folders/...` prefix spends most of that before
/// anything is appended. The socket is not served yet; picking the directory
/// that will still fit when it is costs nothing now.
fn default_data_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".telividb")
        .join("data")
}

/// The loopback port the engine listens on by default.
///
/// Loopback, and only for now. A TCP port on 127.0.0.1 is reachable by every
/// process on the machine, where a Unix socket carries filesystem permissions
/// — which is the transport this is meant to move to.
const DEFAULT_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 7700);
