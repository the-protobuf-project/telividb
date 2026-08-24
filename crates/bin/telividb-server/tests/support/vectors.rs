//! Fixtures shared by the vector-service integration tests.
//!
//! Each test binary compiles this independently, so anything one binary does
//! not use looks dead to it — hence the allow below, which is the standard
//! idiom for shared test support.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use telividb_proto::point::v1::{
    CreatePointRequest, NamedVector, Point, SearchPointsRequest, Vector,
};
use telividb_server::{ServerConfig, serve};

pub const FIELD: &str = "text_bge";
pub const DIM: usize = 4;

/// Start a server on an ephemeral port against `data_dir`.
///
/// Takes the directory rather than making one so a test can stop a server and
/// start another over the same data — which is how restart recovery is
/// observed from outside.
pub async fn start_at(data_dir: PathBuf) -> SocketAddr {
    let (addr, running) = started(data_dir).await;
    // Deliberately leaked: dropping `Running` drops the stop-sender, which
    // resolves the receiver and shuts the server down immediately. A test that
    // never stops its server wants it alive for the whole run.
    std::mem::forget(running);
    addr
}

/// A running server, plus the handle that stops it.
///
/// Needed because `redb` holds an **exclusive** file lock: a second server over
/// the same directory cannot open a collection the first still has open. A test
/// that observes restart behaviour therefore has to stop the first one, not
/// merely stop talking to it.
pub struct Running {
    stop: tokio::sync::oneshot::Sender<()>,
    joined: tokio::task::JoinHandle<()>,
}

impl Running {
    /// Stop the server and wait for it to finish releasing its files.
    pub async fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.joined.await;
    }
}

/// Start a server and keep the means to stop it.
pub async fn started(data_dir: PathBuf) -> (SocketAddr, Running) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
    let addr = listener.local_addr().expect("bound address");
    drop(listener);

    let (stop, stopped) = tokio::sync::oneshot::channel();
    let joined = tokio::spawn(async move {
        let outcome = serve(ServerConfig {
            // Telemetry installs globally once per process, so tests sharing a
            // binary must not each try to install it.
            environment: telividb_telemetry::Environment::Production,
            data_dir,
            shutdown: Some(stopped),
            ..ServerConfig::at(addr)
        })
        .await;
        if let Err(e) = outcome {
            eprintln!("SERVE FAILED: {e}");
        }
    });

    for _ in 0..100 {
        if std::net::TcpStream::connect(addr).is_ok() {
            return (addr, Running { stop, joined });
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("server did not start on {addr}");
}

pub fn wire_vector(values: &[f32]) -> Vector {
    let mut data = Vec::with_capacity(values.len() * 4);
    for v in values {
        data.extend_from_slice(&v.to_le_bytes());
    }
    Vector {
        data: data.into(),
        dimensions: values.len() as i32,
    }
}

pub fn point_with(values: &[f32]) -> Point {
    Point {
        vectors: vec![NamedVector {
            text: String::new(),
            field_id: FIELD.to_owned(),
            vector: Some(wire_vector(values)),
        }],
        ..Default::default()
    }
}

pub fn create(id: &str, values: &[f32]) -> CreatePointRequest {
    CreatePointRequest {
        parent: "collections/media".to_owned(),
        point_id: id.to_owned(),
        point: Some(point_with(values)),
    }
}

pub fn search(query: &[f32], k: i32) -> SearchPointsRequest {
    SearchPointsRequest {
        parent: "collections/media".to_owned(),
        field_id: FIELD.to_owned(),
        query: Some(wire_vector(query)),
        page_size: k,
        ..Default::default()
    }
}
