//! Starting a real server for the integration tests.
//!
//! `dead_code` is allowed because each test binary compiles this module
//! separately, so a helper used by one binary looks unused to the others.
//! That is a property of how Rust builds integration tests, not a sign the
//! helper is unused.
#![allow(dead_code)]

//!
//! Shared by every integration test in this crate, so the two test binaries
//! agree about how a server is started rather than each growing its own
//! slightly different version.

use std::net::SocketAddr;
use std::time::Duration;
use telividb_client::{Client, NewCollection};
use telividb_server::{ServerConfig, serve};

/// Start a server on an ephemeral port and wait until it accepts connections.
pub async fn start() -> (SocketAddr, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp dir");

    // Bind to find a free port, then release it: the server binds it itself,
    // and holding it here would make that fail.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
    let addr = listener.local_addr().expect("bound address");
    drop(listener);

    let data_dir = dir.path().to_path_buf();
    tokio::spawn(async move {
        let outcome = serve(ServerConfig {
            // Telemetry installs globally, once per process. Tests share a
            // binary, so each must not try to install it again.
            environment: telividb_telemetry::Environment::Production,
            data_dir,
            ..ServerConfig::at(addr)
        })
        .await;
        if let Err(e) = outcome {
            eprintln!("SERVE FAILED: {e}");
        }
    });

    for _ in 0..200 {
        if std::net::TcpStream::connect(addr).is_ok() {
            return (addr, dir);
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("server did not start on {addr}");
}

/// Connect an SDK client to a freshly started server.
pub async fn connected() -> (Client, tempfile::TempDir) {
    let (addr, dir) = start().await;
    let client = Client::connect(format!("http://{addr}"))
        .await
        .expect("connect");
    (client, dir)
}

/// Create a collection declaring one text field of `dim` dimensions, and
/// return a handle to it.
///
/// Points cannot be written to a collection that does not exist, so every test
/// starts here — which is the flow a real caller follows too.
pub async fn collection(client: &mut Client, id: &str, dim: usize) -> telividb_client::Collection {
    client
        .create_collection(NewCollection::new(id, descriptor_set()).text_field("text", dim))
        .await
        .expect("create collection");
    client.collection(id)
}

/// A real compiled descriptor set.
///
/// This workspace's own, which is not the schema a production collection would
/// carry — but it is genuinely a `FileDescriptorSet`, which is what the server
/// refuses to create a collection without.
pub fn descriptor_set() -> Vec<u8> {
    telividb_buffers::protobuf::FILE_DESCRIPTOR_SET.to_vec()
}

/// The GGUF the text tests need, if it has been downloaded.
///
/// Looked up rather than required: the file is 80 MiB and not committed, so a
/// fresh clone has none and those tests skip instead of failing.
pub fn model_path() -> Option<std::path::PathBuf> {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/models/gguf/text");
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "gguf"))
}

/// A server holding an embedding model, or `None` when none is downloaded.
pub async fn model_server() -> Option<(Client, tempfile::TempDir)> {
    let model = model_path()?;
    let dir = tempfile::tempdir().expect("temp dir");

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
    let addr = listener.local_addr().expect("bound address");
    drop(listener);

    let data_dir = dir.path().to_path_buf();
    tokio::spawn(async move {
        let outcome = serve(ServerConfig {
            environment: telividb_telemetry::Environment::Production,
            data_dir,
            model_path: Some(model),
            model_name: "nomic-embed-text-v1.5".to_owned(),
            ..ServerConfig::at(addr)
        })
        .await;
        if let Err(e) = outcome {
            eprintln!("SERVE FAILED: {e}");
        }
    });

    // Two waits, because there are two events. The server binds quickly, and
    // *then* loads the model — reading several hundred megabytes and uploading
    // them to a device takes seconds. Loading after binding is deliberate:
    // blocking startup on it made the desktop app give up waiting for its own
    // engine and shut down.
    //
    // So a caller that writes text the instant it connects is refused, which is
    // correct and is what a real client sees too. Waiting for residency here is
    // what every test below would otherwise have to do itself.
    let mut client = None;
    for _ in 0..600 {
        if std::net::TcpStream::connect(addr).is_ok() {
            client = Some(
                Client::connect(format!("http://{addr}"))
                    .await
                    .expect("connect"),
            );
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let mut client = client.unwrap_or_else(|| panic!("server did not start on {addr}"));

    for _ in 0..600 {
        let resident = client
            .list_models()
            .await
            .map(|models| models.iter().any(|m| m.resident))
            .unwrap_or(false);
        if resident {
            return Some((client, dir));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the configured model never became resident on {addr}");
}
