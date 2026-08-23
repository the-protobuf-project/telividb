//! Turning command-line arguments into a [`ServerConfig`].
//!
//! In the library rather than the binary for two reasons: it is configuration
//! logic rather than process logic, and every file under `src/bin/` becomes its
//! own binary target — so a test could not sit beside it there.

use crate::ServerConfig;
use std::net::SocketAddr;

/// Usage text, printed for `--help` and on any argument error.
pub const USAGE: &str = "\
episteme — a multimodal vector and graph database

USAGE:
    episteme [OPTIONS]

OPTIONS:
    --addr <ADDR>          Address to serve gRPC on   [default: 127.0.0.1:7700]
    --metrics <ADDR>       Serve Prometheus metrics   [default: disabled]
    --log <FILTER>         Log filter                 [default: info]
    --log-format <FORMAT>  `text` or `json`           [default: text]
    -h, --help             Print this message
";

/// Parse arguments, returning `None` when help was requested.
///
/// An unrecognised flag is an **error**, not a warning. A server that binds
/// somewhere other than the address it was given is worse than one that refuses
/// to start: the mistake is invisible until something cannot reach it.
pub fn parse(args: impl Iterator<Item = String>) -> Result<Option<ServerConfig>, String> {
    let mut config = ServerConfig::default();
    let mut args = args.peekable();

    while let Some(flag) = args.next() {
        // A bare `--` separates cargo's arguments from the binary's. It reaches
        // here whenever someone writes `cargo serve -- --addr ...`, and it
        // means nothing to us.
        if flag == "--" {
            continue;
        }
        if flag == "-h" || flag == "--help" {
            return Ok(None);
        }

        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;

        match flag.as_str() {
            "--addr" => {
                config.addr = value
                    .parse::<SocketAddr>()
                    .map_err(|e| format!("--addr {value:?}: {e}"))?;
            }
            "--metrics" => {
                config.metrics_addr = Some(
                    value
                        .parse::<SocketAddr>()
                        .map_err(|e| format!("--metrics {value:?}: {e}"))?,
                );
            }
            "--log" => config.log_filter = value,
            "--log-format" => match value.as_str() {
                "json" => config.log_json = true,
                "text" => config.log_json = false,
                other => return Err(format!("--log-format {other:?}: expected `text` or `json`")),
            },
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(Some(config))
}

#[cfg(test)]
#[path = "args_test.rs"]
mod tests;
