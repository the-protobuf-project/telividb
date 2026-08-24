//! Turning command-line arguments into a [`ServerConfig`].
//!
//! In the library rather than the binary for two reasons: it is configuration
//! logic rather than process logic, and every file under `src/bin/` becomes its
//! own binary target — so a test could not sit beside it there.

use crate::ServerConfig;
use std::net::SocketAddr;
use telividb_telemetry::{Environment, LogLevel};

/// Usage text, printed for `--help` and on any argument error.
pub const USAGE: &str = "\
telividb — a multimodal vector and graph database

USAGE:
    telividb [OPTIONS]

OPTIONS:
    --addr <ADDR>          Address to serve gRPC on   [default: 127.0.0.1:7700]
    --otlp <ADDR>          Export to an OTLP collector [default: console only]
    --mcap <PATH>          Record an MCAP file         [default: disabled]
    --environment <ENV>    development | staging | production | jetson
                           [default: development]
    --log-level <LEVEL>    error | info | debug
                           [default: from telemetry.toml]
    --telemetry-config <PATH>
                           Path to telemetry.toml   [default: discovered by CWD]
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
            "--otlp" => {
                config.otlp_addr = Some(
                    value
                        .parse::<SocketAddr>()
                        .map_err(|e| format!("--otlp {value:?}: {e}"))?,
                );
            }
            "--mcap" => config.mcap_path = Some(value.into()),
            "--telemetry-config" => config.telemetry_config = Some(value.into()),
            "--environment" => config.environment = environment_of(&value)?,
            "--log-level" => config.log_level = Some(log_level_of(&value)?),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(Some(config))
}

/// Map an environment name onto the stack's enum.
///
/// The single string-to-enum conversion in the server. Doing it here rather
/// than at use means an unknown value is rejected while the operator is still
/// looking at the terminal, instead of falling through to a silent default
/// somewhere the mistake is invisible.
fn environment_of(value: &str) -> Result<Environment, String> {
    match value {
        "development" => Ok(Environment::Development),
        "staging" => Ok(Environment::Staging),
        "production" => Ok(Environment::Production),
        "jetson" => Ok(Environment::Jetson),
        other => Err(format!(
            "--environment {other:?}: expected development, staging, \
             production or jetson"
        )),
    }
}

/// Map a verbosity name onto the stack's enum.
fn log_level_of(value: &str) -> Result<LogLevel, String> {
    match value {
        "error" => Ok(LogLevel::ModuleLevel_1),
        "info" => Ok(LogLevel::ModuleLevel_2),
        "debug" => Ok(LogLevel::ModuleLevel_3),
        other => Err(format!(
            "--log-level {other:?}: expected error, info or debug"
        )),
    }
}

#[cfg(test)]
#[path = "args_test.rs"]
mod tests;
