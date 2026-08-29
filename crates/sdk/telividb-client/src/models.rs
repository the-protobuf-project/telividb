//! The model catalog, from a client's side.
//!
//! Thin on purpose: every decision — which architectures load, what a digest
//! must match, whether a modality has an encoder — belongs to the server, so
//! this moves messages and nothing else. A window and `grpcurl` therefore see
//! the same answers, which is the property that keeps the desktop app an
//! ordinary client (rule 16).

use crate::{Client, Result};
use telividb_buffers::protobuf::models::v1 as wire;

impl Client {
    /// Every model on offer.
    pub async fn list_models(&mut self) -> Result<Vec<wire::CatalogModel>> {
        let response = self
            .models
            .list_catalog_models(wire::ListCatalogModelsRequest::default())
            .await?
            .into_inner();
        Ok(response.catalog_models)
    }

    /// One model by catalog id.
    pub async fn get_model(&mut self, id: &str) -> Result<wire::CatalogModel> {
        Ok(self
            .models
            .get_catalog_model(wire::GetCatalogModelRequest {
                name: format!("catalogModels/{id}"),
            })
            .await?
            .into_inner())
    }

    /// Begin installing a model, returning the handle to follow it by.
    ///
    /// Returns as soon as the work is accepted. Calling it twice for one model
    /// returns the running installation rather than starting a second transfer,
    /// so a caller need not guard the button.
    pub async fn install_model(&mut self, id: &str) -> Result<wire::ModelInstallation> {
        Ok(self
            .models
            .create_model_installation(wire::CreateModelInstallationRequest {
                model_installation: Some(wire::ModelInstallation {
                    catalog_model: format!("catalogModels/{id}"),
                    ..Default::default()
                }),
                model_installation_id: String::new(),
            })
            .await?
            .into_inner())
    }

    /// How far an installation has got.
    pub async fn installation(&mut self, name: &str) -> Result<wire::ModelInstallation> {
        Ok(self
            .models
            .get_model_installation(wire::GetModelInstallationRequest {
                name: name.to_owned(),
            })
            .await?
            .into_inner())
    }

    /// Stop an installation.
    ///
    /// The partial file is kept, so installing again resumes rather than
    /// starting over — which on a model measured in hundreds of megabytes is
    /// the difference between a pause and an hour.
    pub async fn cancel_installation(&mut self, name: &str) -> Result<wire::ModelInstallation> {
        Ok(self
            .models
            .delete_model_installation(wire::DeleteModelInstallationRequest {
                name: name.to_owned(),
            })
            .await?
            .into_inner())
    }
}
