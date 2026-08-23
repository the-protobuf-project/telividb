//! Starting the server.

use crate::config::ServerConfig;
use crate::error::{Error, Result};
use crate::services::CollectionSvc;
use episteme_proto::FILE_DESCRIPTOR_SET;
use episteme_proto::collection::v1::collections_server::CollectionsServer;
use episteme_telemetry::{Telemetry, TelemetryConfig};

/// Install telemetry, build the router, and serve until shutdown.
///
/// Telemetry is installed here rather than in any library, because a library
/// that installs a subscriber decides for every binary that ever links it.
pub async fn serve(config: ServerConfig) -> Result<()> {
    // An already-installed pipeline is not fatal. An embedded caller may have
    // configured its own subscriber before starting a server, and refusing to
    // serve because logging is already set up would be the wrong call — the
    // caller's choice wins, and the server says so rather than dying.
    let _telemetry = match Telemetry::install(TelemetryConfig {
        filter: config.log_filter.clone(),
        json: config.log_json,
        prometheus: config.metrics_addr,
        recall_sample_rate: 0.0,
    }) {
        Ok(t) => Some(t),
        Err(episteme_telemetry::init::TelemetryError::AlreadyInstalled(_)) => {
            tracing::debug!("telemetry already installed; using the existing pipeline");
            None
        }
        Err(e) => return Err(Error::Telemetry(e.to_string())),
    };

    // Health reports SERVING once the router is up. A load balancer needs this
    // to distinguish "starting" from "broken", and it costs one service.
    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<CollectionsServer<CollectionSvc>>()
        .await;

    let mut router = tonic::service::Routes::default()
        .add_service(health_service)
        .add_service(CollectionsServer::new(CollectionSvc::default()));

    if config.reflection {
        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| Error::Reflection(e.to_string()))?;
        router = router.add_service(reflection);
    }

    tracing::info!(
        addr = %config.addr,
        grpc_web = config.grpc_web,
        reflection = config.reflection,
        "episteme listening"
    );

    let mut server = tonic::transport::Server::builder();

    // gRPC-web is a `tower` layer, not a second API — but it has to be in place
    // from the start, because it constrains streaming: server-streaming works
    // over gRPC-web, bidirectional does not.
    if config.grpc_web {
        server
            .accept_http1(true)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_routes(router)
            .serve_with_shutdown(config.addr, shutdown())
            .await
            .map_err(|e| Error::Transport(e.to_string()))
    } else {
        server
            .add_routes(router)
            .serve_with_shutdown(config.addr, shutdown())
            .await
            .map_err(|e| Error::Transport(e.to_string()))
    }
}

/// Resolve when the process is asked to stop.
///
/// Graceful shutdown matters more here than in most servers: a sealed segment
/// half-written is a temp directory, but an interrupted manifest swap would be
/// the one moment a collection is mid-publish.
async fn shutdown() {
    if tokio::signal::ctrl_c().await.is_ok() {
        tracing::info!("shutdown requested");
    }
}
