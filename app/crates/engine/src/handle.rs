//! Starting the engine, and stopping it again.

use crate::{DataDirLock, Error, Result};
use std::net::SocketAddr;
use std::path::PathBuf;
use telividb_client::Client;
use telividb_server::ServerConfig;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// A running engine and a client that reaches it.
///
/// Dropping this stops the server: the shutdown sender goes with it, and
/// `serve` treats a dropped sender as a stop. So quitting the window shuts the
/// engine down through the same path a deliberate stop uses, rather than
/// leaving it orphaned for the process to kill.
pub struct Engine {
    /// Sending — or dropping — this stops the server.
    stop: Option<oneshot::Sender<()>>,
    /// The server task, so a caller can wait for it to finish.
    serving: Option<JoinHandle<()>>,
    /// Where the server is listening, for a second client or an external tool.
    addr: SocketAddr,
    /// A connected client. Cheap to clone; `Channel` multiplexes.
    client: Client,
    /// The claim on the data directory.
    ///
    /// Declared last on purpose: struct fields drop in declaration order, so
    /// this is released after the shutdown signal above rather than before it.
    /// The other order hands the directory to a second instance while the first
    /// is still draining requests and flushing telemetry — which is the one
    /// thing the claim exists to prevent.
    ///
    /// Dropping still does not *wait*. Use [`Engine::shutdown`] where an await
    /// is possible; this ordering is what makes the synchronous path merely
    /// racy rather than reliably wrong.
    _lock: DataDirLock,
}

impl Engine {
    /// Claim `data_dir`, start the engine on `addr`, and connect to it.
    ///
    /// Call once per process. Telemetry installs globally and exactly once, and
    /// a failed install is fatal by design — a server whose telemetry silently
    /// did not start is one nobody can debug afterwards.
    pub async fn start(
        data_dir: PathBuf,
        addr: SocketAddr,
        model: Option<PathBuf>,
    ) -> Result<Self> {
        // Before the engine opens anything. A busy directory should be a
        // sentence about another window, not a storage error.
        let lock = DataDirLock::acquire(&data_dir)?;

        let (stop, shutdown) = oneshot::channel();
        let config = ServerConfig {
            data_dir,
            model_path: model,
            shutdown: Some(shutdown),
            ..ServerConfig::at(addr)
        };

        let (ready, started) = oneshot::channel();
        let serving = tokio::spawn(async move {
            let outcome = telividb_server::serve(config).await;
            // Only observed when the server stops before anything connects;
            // afterwards the receiver is gone and the send fails harmlessly.
            let _ = ready.send(outcome.err().map(|e| e.to_string()));
        });

        // Not `?`. Returning here would drop `stop` and `lock` together with
        // the frame: the shutdown signal would be sent and the claim on the
        // data directory released in the same instant, while the server is
        // still unwinding. A caller that retried immediately — which is the
        // obvious thing to do when a connection fails — would meet a directory
        // that looks free and is not.
        //
        // So the failure path waits for the same thing the success path does.
        // `lock` is still live here and drops at the end of this function,
        // after the await.
        let client = match connect(addr, started).await {
            Ok(client) => client,
            Err(error) => {
                let _ = stop.send(());
                let _ = serving.await;
                return Err(error);
            }
        };

        Ok(Self {
            stop: Some(stop),
            serving: Some(serving),
            addr,
            client,
            _lock: lock,
        })
    }

    /// A client reaching this engine.
    pub fn client(&self) -> Client {
        self.client.clone()
    }

    /// Where the engine is listening.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Stop the engine and wait for it to finish.
    ///
    /// Signals shutdown, waits for the server task to return, and only then
    /// releases the claim on the data directory — so when this returns, the
    /// directory is genuinely free and the next instance will not find a
    /// half-flushed one.
    ///
    /// Dropping an `Engine` still stops the server, because dropping the sender
    /// is itself the signal. What dropping cannot do is wait.
    pub async fn shutdown(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(serving) = self.serving.take() {
            // A panic in the server task is already reported by its own
            // telemetry; there is nothing useful to do with the error here
            // except decline to hang on it.
            let _ = serving.await;
        }
    }
}

/// Connect once the server is accepting, or report why it never did.
///
/// The server binds its listener inside `serve`, so a client that connects the
/// instant the task is spawned can beat it. Retrying briefly is what makes
/// startup deterministic; racing the failure channel is what turns "connection
/// refused" into the server's own reason for stopping.
async fn connect(
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
const RETRIES: usize = 50;

/// How long to wait between attempts — 50 × 20 ms is a second of patience.
const RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(20);
