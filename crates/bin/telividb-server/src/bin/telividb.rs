//! The telividb daemon.
//!
//! Headless by design. On macOS this runs natively under launchd rather than in
//! a container, because Apple GPUs have no IOMMU and `Hypervisor.framework`
//! exposes no virtual GPU — so a container gets no Metal.

use std::process::ExitCode;
use telividb_server::args::{USAGE, parse};
use telividb_server::serve;

fn main() -> ExitCode {
    let config = match parse(std::env::args().skip(1)) {
        Ok(Some(config)) => config,
        Ok(None) => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("telividb: {message}\n");
            eprint!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("telividb: cannot start the async runtime: {e}");
            return ExitCode::FAILURE;
        }
    };

    match runtime.block_on(serve(config)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("telividb: {e}");
            ExitCode::FAILURE
        }
    }
}
