//! The episteme daemon.
//!
//! Headless by design. On macOS this runs natively under launchd rather than in
//! a container, because Apple GPUs have no IOMMU and `Hypervisor.framework`
//! exposes no virtual GPU — so a container gets no Metal.

use episteme_server::{ServerConfig, serve};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ServerConfig::default();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i + 1 < args.len() {
        let value = &args[i + 1];
        match args[i].as_str() {
            "--addr" => config.addr = value.parse::<SocketAddr>()?,
            "--metrics" => config.metrics_addr = Some(value.parse::<SocketAddr>()?),
            "--log" => config.log_filter = value.clone(),
            "--log-format" => config.log_json = value == "json",
            other => eprintln!("ignoring unknown flag {other}"),
        }
        i += 2;
    }

    serve(config).await?;
    Ok(())
}
