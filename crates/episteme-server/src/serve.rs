//! Starting the server.

use crate::config::ServerConfig;
use crate::error::{Error, Result};
use crate::services::CollectionSvc;
use episteme_proto::FILE_DESCRIPTOR_SET;
use episteme_proto::collection::v1::collections_server::CollectionsServer;
use episteme_telemetry::{Telemetry, TelemetryConfig, logger};

/// Install telemetry, build the router, and serve until shutdown.
///
/// Telemetry is installed here rather than in any library, because a library
/// that installs a subscriber decides for every binary that ever links it.
pub async fn serve(config: ServerConfig) -> Result<()> {
    // A failure here is fatal, and deliberately so. The previous behaviour
    // treated every error as "already installed" and reported it through a
    // facade that, in the failure case, has no pipeline behind it — so an
    // unwritable `--mcap` path or an unreachable collector started a server
    // with no telemetry and no diagnostic anywhere. `install` distinguishes a
    // real build failure from a benign one; this propagates the real ones.
    let telemetry = Telemetry::install(TelemetryConfig {
        environment: config.environment.clone(),
        log_level: config.log_level,
        otlp: config.otlp_addr,
        mcap_path: config.mcap_path.clone(),
        config_path: config.telemetry_config.clone(),
        ..TelemetryConfig::default()
    })
    .map_err(|e| Error::Telemetry(e.to_string()))?;

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

    announce(&config);

    let listener = tokio::net::TcpListener::bind(config.addr)
        .await
        .map_err(|source| Error::Bind {
            addr: config.addr,
            source,
        })?;
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

    let mut server = tonic::transport::Server::builder();

    // gRPC-web is a `tower` layer, not a second API — but it has to be in place
    // from the start, because it constrains streaming: server-streaming works
    // over gRPC-web, bidirectional does not.
    let served = if config.grpc_web {
        server
            .accept_http1(true)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_routes(router)
            .serve_with_incoming_shutdown(incoming, shutdown())
            .await
    } else {
        server
            .add_routes(router)
            .serve_with_incoming_shutdown(incoming, shutdown())
            .await
    };

    // Flush before returning: an exporter batches, so the last few seconds of a
    // run are otherwise lost exactly when something has gone wrong enough to
    // stop the process. Best-effort, because with no collector configured the
    // stack's meter has nothing to flush and says so — which is not a reason to
    // fail a clean shutdown.
    let _ = telemetry.flush();

    served.map_err(|e| Error::Transport(e.to_string()))
}

/// Say where every stream of telemetry goes, at startup, every time.
///
/// The alternative is what this replaced: a server that logs one line and then
/// appears silent, with no indication that the instrumentation exists, that
/// metrics are off, or where a log file would be if there were one.
fn announce(config: &ServerConfig) {
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
}

/// Resolve when the process is asked to stop.
///
/// Graceful shutdown matters more here than in most servers: a sealed segment
/// half-written is a temp directory, but an interrupted manifest swap would be
/// the one moment a collection is mid-publish.
async fn shutdown() {
    // A failure to *install* the handler must not resolve: `ctrl_c` returning
    // `Err` would otherwise trigger an immediate graceful shutdown, so a server
    // that could not register a signal handler would exit the instant it
    // started serving. Never shutting down is the safe direction — the process
    // can still be killed.
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            logger::info!("shutdown requested");
        }
        Err(error) => {
            logger::error!("cannot install the ctrl-c handler: {error}");
            std::future::pending::<()>().await;
        }
    }
}
