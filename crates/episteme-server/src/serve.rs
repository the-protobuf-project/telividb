//! Starting the server.

use crate::config::ServerConfig;
use crate::error::{Error, Result};
use crate::services::CollectionSvc;
use episteme_proto::FILE_DESCRIPTOR_SET;
use episteme_proto::collection::v1::collections_server::CollectionsServer;
use episteme_telemetry::{Environment, Telemetry, TelemetryConfig, logger};

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
        environment: environment_of(&config.environment),
        otlp: config.otlp_addr,
        mcap_path: config.mcap_path.as_ref().map(|p| p.display().to_string()),
        ..TelemetryConfig::default()
    }) {
        Ok(t) => Some(t),
        // An already-installed pipeline is not fatal. An embedded caller may
        // have configured its own before starting a server, and refusing to
        // serve because logging is already set up would be the wrong call —
        // the caller's choice wins, and the server says so rather than dying.
        Err(e) => {
            tracing::debug!(%e, "telemetry already installed; using the existing pipeline");
            None
        }
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
            // Health has to be registered too, or it is routed but not
            // discoverable: a client that finds services by reflection — which
            // is every generic client, `grpcurl` included — reports that the
            // server does not expose it at all.
            .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| Error::Reflection(e.to_string()))?;
        router = router.add_service(reflection);
    }

    // Say where every stream of telemetry goes, at startup, every time.
    //
    // The alternative is what this replaced: a server that logs one line and
    // then appears silent, with no indication that the instrumentation exists,
    // that metrics are off, or where a log file would be if there were one.
    logger::info!(
        "episteme listening on {} (grpc-web {}, reflection {})",
        config.addr,
        config.grpc_web,
        config.reflection
    );
    // Each macro returns a `LogBuilder` that emits when it drops — hence the
    // blocks: a bare match arm would make the arms' types the builder rather
    // than `()`.
    match config.otlp_addr {
        Some(addr) => {
            logger::info!("telemetry: exporting logs, traces and metrics to {addr}");
        }
        None => {
            logger::info!("telemetry: console only — pass --otlp <addr> to export");
        }
    }
    if let Some(path) = &config.mcap_path {
        logger::info!(
            "telemetry: recording MCAP for Foxglove at {}",
            path.display()
        );
    }
    logger::info!("telemetry: environment {}", config.environment);

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

/// Map the configured name onto the stack's environment.
///
/// Validated at parse time, so an unknown value cannot reach here — but the
/// fallback is `Development` rather than a panic, because a server refusing to
/// start over a logging label would be the wrong trade.
fn environment_of(name: &str) -> Environment {
    match name {
        "production" => Environment::Production,
        "staging" => Environment::Staging,
        "jetson" => Environment::Jetson,
        _ => Environment::Development,
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
