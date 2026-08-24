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
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
    let addr = listener.local_addr().expect("bound address");
    drop(listener);

    tokio::spawn(async move {
        let outcome = serve(ServerConfig {
            // Telemetry installs globally once per process, so tests sharing a
            // binary must not each try to install it.
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
            return addr;
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
