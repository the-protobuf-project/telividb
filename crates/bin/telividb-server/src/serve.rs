//! Starting the server.

use crate::announce::{announce, announce_residency};
use crate::config::ServerConfig;
use crate::error::{Error, Result};
use crate::services::{CollectionSvc, Embeddings, PointsSvc};
use telividb_buffers::protobuf::FILE_DESCRIPTOR_SET;
use telividb_buffers::protobuf::collection::v1::collections_server::CollectionsServer;
use telividb_buffers::protobuf::models::v1::models_server::ModelsServer;
use telividb_buffers::protobuf::point::v1::points_server::PointsServer;
use telividb_buffers::protobuf::tenancy::v1::organizations_server::OrganizationsServer;
use telividb_buffers::protobuf::tenancy::v1::projects_server::ProjectsServer;
use telividb_buffers::protobuf::tenancy::v1::sessions_server::SessionsServer;
use telividb_buffers::protobuf::tenancy::v1::spaces_server::SpacesServer;
use telividb_telemetry::{Telemetry, TelemetryConfig, fields, logger};

/// Install telemetry, build the router, and serve until shutdown.
///
/// Telemetry is installed here rather than in any library, because a library
/// that installs a subscriber decides for every binary that ever links it.
pub async fn serve(mut config: ServerConfig) -> Result<()> {
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

    // One catalogue, shared: the collection service owns it and the point
    // service consults it, so the two cannot disagree about what exists.
    let collections = CollectionSvc::open(config.data_dir.clone())
        .map_err(|e| Error::Catalogue(e.to_string()))?;
    let points = build_points(&config)?.with_catalogue(collections.catalogue());

    // Health reports SERVING once the router is up. A load balancer needs this
    // to distinguish "starting" from "broken", and it costs one service.
    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<CollectionsServer<CollectionSvc>>()
        .await;
    health_reporter
        .set_serving::<PointsServer<PointsSvc>>()
        .await;

    // Same failure mode as the catalogue, and the same cause worth naming: a
    // second server pointed at a data directory one already holds.
    let tenancy = crate::services::tenancy::TenancySvc::open(&config.data_dir)
        .map_err(|e| Error::Catalogue(e.to_string()))?;

    let mut router = tonic::service::Routes::default()
        .add_service(health_service)
        .add_service(CollectionsServer::new(collections))
        .add_service(PointsServer::new(points))
        // Four services over one store. They share a `redb` file because they
        // are one tree, and sharing it is what lets a delete and the read that
        // follows it agree.
        .add_service(OrganizationsServer::new(tenancy.clone()))
        .add_service(ProjectsServer::new(tenancy.clone()))
        .add_service(SpacesServer::new(tenancy.clone()))
        .add_service(SessionsServer::new(tenancy))
        // The catalog. Stateless apart from the model directory, so it is
        // built here rather than opened — nothing to lock, nothing to fail.
        .add_service(ModelsServer::new(crate::services::models::ModelsSvc::new(
            &config.data_dir,
        )));

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

    let listener = tokio::net::TcpListener::bind(config.addr)
        .await
        .map_err(|source| Error::Bind {
            addr: config.addr,
            source,
        })?;

    let stop = config.shutdown.take();
    announce(&config);

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
            .serve_with_incoming_shutdown(incoming, shutdown(stop))
            .await
    } else {
        server
            .add_routes(router)
            .serve_with_incoming_shutdown(incoming, shutdown(stop))
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

/// Resolve when the process is asked to stop.
///
/// Graceful shutdown matters more here than in most servers: a sealed segment
/// half-written is a temp directory, but an interrupted manifest swap would be
/// the one moment a collection is mid-publish.
async fn shutdown(stop: Option<tokio::sync::oneshot::Receiver<()>>) {
    // Either signal ends the server. A dropped sender counts as a stop, so a
    // caller that goes away does not leave the server running forever.
    if let Some(stop) = stop {
        tokio::select! {
            _ = stop => {
                logger::info!("shutdown requested");
                announce_residency();
                return;
            }
            _ = ctrl_c() => return,
        }
    }
    ctrl_c().await
}

/// Wait for ctrl-c, then report what was resident.
async fn ctrl_c() {
    // A failure to *install* the handler must not resolve: `ctrl_c` returning
    // `Err` would otherwise trigger an immediate graceful shutdown, so a server
    // that could not register a signal handler would exit the instant it
    // started serving. Never shutting down is the safe direction — the process
    // can still be killed.
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            logger::info!("shutdown requested");
            announce_residency();
        }
        Err(error) => {
            logger::error!("cannot install the ctrl-c handler: {error}");
            std::future::pending::<()>().await;
        }
    }
}

/// Build the point service, loading an embedding model if one was configured.
///
/// Loaded here, at startup, rather than on first use: rule 45 holds models
/// resident, and a first request that blocks for several seconds on a load is
/// indistinguishable from one that has hung. A failure to load is fatal for
/// the same reason — a server that started without the model it was told to
/// serve would refuse every text request while looking healthy.
fn build_points(config: &ServerConfig) -> Result<PointsSvc> {
    let svc = PointsSvc::new(config.data_dir.clone());

    let Some(path) = &config.model_path else {
        logger::info!("no embedding model configured").with_data(&serde_json::json!({
            fields::STRATEGY: "vectors-only",
        }));
        return Ok(svc);
    };

    let embeddings = Embeddings::load(path, &config.model_name)
        .map_err(|e| Error::Model(format!("{}: {e}", path.display())))?;
    Ok(svc.with_embeddings(embeddings))
}
