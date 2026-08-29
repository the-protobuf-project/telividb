//! The model catalog over real gRPC.
//!
//! Everything here runs against the compiled-in catalog and touches no network:
//! listing, filtering and refusals are all decided before a byte moves. The
//! transfer itself is covered by `telividb-models`' own live tests, which are
//! `#[ignore]`d for the reason CI must not depend on a third party being up.

mod support;

use support::server::TestServer;
use telividb_buffers::protobuf::models::v1::models_client::ModelsClient;
use telividb_buffers::protobuf::models::v1::{
    CreateModelInstallationRequest, DeleteModelInstallationRequest, GetCatalogModelRequest,
    GetModelInstallationRequest, InstallationState, ListCatalogModelsRequest, Modality,
    ModelInstallation,
};

#[tokio::test]
async fn the_catalog_is_offered_with_everything_needed_to_choose() {
    let server = TestServer::start().await;
    let mut models = ModelsClient::connect(server.url()).await.expect("connect");

    let listed = models
        .list_catalog_models(ListCatalogModelsRequest::default())
        .await
        .expect("list")
        .into_inner();

    assert!(!listed.catalog_models.is_empty());
    for model in &listed.catalog_models {
        assert!(model.name.starts_with("catalogModels/"), "{}", model.name);
        // The fields a person actually chooses on. A catalog entry missing any
        // of them is a row in a table with a blank cell.
        assert!(!model.display_name.is_empty(), "{}", model.name);
        assert!(!model.description.is_empty(), "{}", model.name);
        assert_eq!(model.digest.len(), 64, "{}", model.name);
        assert!(model.size_bytes > 1_000_000, "{}", model.name);
        assert!(model.dimensions > 0, "{}", model.name);
        assert!(
            model.repository_uri.starts_with("https://"),
            "{}: the link a person follows to check the licence",
            model.name
        );
        assert_eq!(model.modality, Modality::Text as i32, "{}", model.name);
    }

    let recommended: Vec<&str> = listed
        .catalog_models
        .iter()
        .filter(|m| m.recommended)
        .map(|m| m.name.as_str())
        .collect();
    assert_eq!(recommended.len(), 1, "one default, not none and not two");
}

#[tokio::test]
async fn a_modality_with_no_encoder_lists_empty_rather_than_erroring() {
    // Rule 49's posture applied to a listing: asking for audio models is a
    // reasonable question with an honest answer of "none", which the window can
    // explain. An error would read as a fault in the request.
    let server = TestServer::start().await;
    let mut models = ModelsClient::connect(server.url()).await.expect("connect");

    let audio = models
        .list_catalog_models(ListCatalogModelsRequest {
            filter: "modality = \"audio\"".to_owned(),
            ..Default::default()
        })
        .await
        .expect("audio is a valid question")
        .into_inner();
    assert!(audio.catalog_models.is_empty());

    let text = models
        .list_catalog_models(ListCatalogModelsRequest {
            filter: "modality = \"text\"".to_owned(),
            ..Default::default()
        })
        .await
        .expect("text")
        .into_inner();
    assert!(!text.catalog_models.is_empty());
}

#[tokio::test]
async fn a_model_can_be_fetched_by_name_and_an_unknown_one_says_so() {
    let server = TestServer::start().await;
    let mut models = ModelsClient::connect(server.url()).await.expect("connect");

    let one = models
        .get_catalog_model(GetCatalogModelRequest {
            name: "catalogModels/bge-small-en-v1.5".to_owned(),
        })
        .await
        .expect("get")
        .into_inner();
    assert_eq!(one.architecture, "bert");
    assert_eq!(one.dimensions, 384);
    assert!(
        !one.installed,
        "a fresh data directory has nothing installed"
    );

    let status = models
        .get_catalog_model(GetCatalogModelRequest {
            name: "catalogModels/no-such-model".to_owned(),
        })
        .await
        .expect_err("an unknown model");
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert!(status.message().contains("no-such-model"));
}

#[tokio::test]
async fn installing_an_unknown_model_is_refused_before_anything_starts() {
    let server = TestServer::start().await;
    let mut models = ModelsClient::connect(server.url()).await.expect("connect");

    let status = models
        .create_model_installation(CreateModelInstallationRequest {
            model_installation: Some(ModelInstallation {
                catalog_model: "catalogModels/not-a-model".to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect_err("nothing to install");
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn an_installation_is_a_handle_and_asking_twice_does_not_start_two() {
    // The property that makes this safe to call from a button: a second click
    // returns the running transfer rather than starting a second one into the
    // same path.
    let server = TestServer::start().await;
    let mut models = ModelsClient::connect(server.url()).await.expect("connect");

    let request = CreateModelInstallationRequest {
        model_installation: Some(ModelInstallation {
            catalog_model: "catalogModels/bge-small-en-v1.5".to_owned(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let first = models
        .create_model_installation(request.clone())
        .await
        .expect("accepted")
        .into_inner();
    assert!(first.name.starts_with("modelInstallations/"));
    assert_eq!(
        first.total_bytes, 36_806_944,
        "the size a progress bar divides by"
    );
    assert!(first.create_time.is_some());

    let second = models
        .create_model_installation(request)
        .await
        .expect("accepted again")
        .into_inner();
    assert_eq!(
        second.name, first.name,
        "a second call started a second transfer"
    );

    // Cancelling leaves it cancellable-and-resumable rather than deleted: the
    // partial file is kept deliberately.
    let cancelled = models
        .delete_model_installation(DeleteModelInstallationRequest {
            name: first.name.clone(),
        })
        .await
        .expect("cancel")
        .into_inner();
    assert!(
        cancelled.state == InstallationState::Cancelled as i32
            || cancelled.state == InstallationState::Succeeded as i32
            || cancelled.state == InstallationState::Failed as i32,
        "state was {}",
        cancelled.state
    );

    // And it is still readable afterwards, so a window can show why it stopped.
    let after = models
        .get_model_installation(GetModelInstallationRequest { name: first.name })
        .await
        .expect("still there")
        .into_inner();
    assert!(after.update_time.is_some());
}
