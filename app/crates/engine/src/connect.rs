//! Waiting for a freshly started engine to accept connections.
//!
//! Split from `handle.rs` because that file is about the engine's lifetime —
//! claiming the directory and the port, starting, stopping — and this is about
//! one moment in it. The budget below is the interesting part, and it earned
//! its own file by being wrong once: too short a wait did not degrade, it
//! stopped the whole application from launching.

use crate::{Error, Result};
use std::net::SocketAddr;
use telividb_client::Client;
use tokio::sync::oneshot;

/// Connect once the server is accepting, or report why it never did.
///
/// The server binds its listener inside `serve`, so a client that connects the
/// instant the task is spawned can beat it. Retrying briefly is what makes
/// startup deterministic; racing the failure channel is what turns "connection
/// refused" into the server's own reason for stopping.
pub(crate) async fn connect(
    addr: SocketAddr,
    mut started: oneshot::Receiver<Option<String>>,
) -> Result<Client> {
    let endpoint = format!("http://{addr}");
    let mut last = None;

    for _ in 0..RETRIES {
        if let Ok(Some(reason)) = started.try_recv() {
            return Err(Error::Startup(reason));
        }
        match Client::connect(endpoint.clone()).await {
            Ok(client) => return Ok(client),
            Err(err) => last = Some(err),
        }
        tokio::time::sleep(RETRY_DELAY).await;
    }

    match started.try_recv() {
        Ok(Some(reason)) => Err(Error::Startup(reason)),
        _ => Err(last.map(Error::Connect).unwrap_or_else(|| {
            Error::Startup("the engine never accepted a connection".to_owned())
        })),
    }
}

/// How many times to retry the first connection.
const RETRIES: usize = 250;

/// How long to wait between attempts — 250 × 20 ms is five seconds of patience.
///
/// It was one second, and one second was not enough: startup also initialises
/// the compute backend, which compiles twenty Metal libraries before anything
/// binds. Exceeding the budget did not degrade gracefully — the app reported
/// "could not reach the engine" and shut down, so the whole product failed to
/// launch over a slow-but-normal start.
///
/// Five seconds is still a bound rather than a wait: the loop exits the moment
/// the engine's own failure arrives on `started`, so a server that will never
/// come up is reported immediately with its reason rather than after the full
/// budget.
const RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(20);
