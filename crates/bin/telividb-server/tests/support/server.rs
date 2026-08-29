//! A running server, for a test that needs to speak real gRPC to one.

// Each test binary compiles the whole `support` module but uses part of it.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::time::Duration;
use telividb_server::{ServerConfig, serve};

/// A server listening on an ephemeral port, with a data directory of its own.
///
/// The directory is per-server rather than shared because the catalogue is a
/// `redb` file and `redb` takes an exclusive lock — two servers under one data
/// directory collide, which under the default `./data` is every test in a
/// binary.
pub struct TestServer {
    /// Where it is listening. Ephemeral, so tests can run in parallel.
    addr: SocketAddr,
    /// Held rather than `keep()`-ed, so dropping this removes the directory.
    ///
    /// Three hand-written copies of this harness called `.keep()`, which is the
    /// easy way to satisfy the borrow checker and leaks a directory per test
    /// into the system temp directory forever. Nothing fails when it happens,
    /// which is why it survived being written three times.
    _dir: tempfile::TempDir,
}

impl TestServer {
    /// Start a server and return once it accepts a connection.
    ///
    /// Panics rather than returning an error: a harness that could not start
    /// has nothing useful to say to a test, and the failure is the test's.
    pub async fn start() -> Self {
        Self::start_with(|_| {}).await
    }

    /// The same, after `prepare` has arranged the data directory.
    ///
    /// For tests about what the server *finds* rather than what it is told —
    /// an installed model, say, which is discovered from the directory rather
    /// than configured.
    pub async fn start_with(prepare: impl FnOnce(&std::path::Path)) -> Self {
        let dir = tempfile::tempdir().expect("temp data dir");
        prepare(dir.path());

        // Bound and released to reserve a free port. A race with another
        // process is possible in principle and has not been worth a retry
        // loop — the window is microseconds and the alternative is a harness
        // more complicated than the tests using it.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
        let addr = listener.local_addr().expect("bound address");
        drop(listener);

        let data_dir = dir.path().to_path_buf();
        tokio::spawn(async move {
            let outcome = serve(ServerConfig {
                // Telemetry installs a *global* logger and only once per
                // process, so tests sharing a binary must not each install one.
                environment: telividb_telemetry::Environment::Production,
                data_dir,
                ..ServerConfig::at(addr)
            })
            .await;
            if let Err(e) = outcome {
                eprintln!("SERVE FAILED: {e}");
            }
        });

        for _ in 0..100 {
            if std::net::TcpStream::connect(addr).is_ok() {
                return Self { addr, _dir: dir };
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("server never accepted a connection on {addr}");
    }

    /// The address it is listening on.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Its origin, in the form a `tonic` client's `connect` wants.
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }
}
