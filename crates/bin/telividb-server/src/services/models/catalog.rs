//! Reading the catalog.

use super::{ModelsSvc, catalog_id, convert};
use telividb_buffers::protobuf::models::v1::{
    CatalogModel, GetCatalogModelRequest, ListCatalogModelsRequest, ListCatalogModelsResponse,
};
use tonic::{Request, Response, Status};

impl ModelsSvc {
    /// Every model on offer, in catalog order.
    ///
    /// Unpaged in practice: the catalog is a curated list of a handful of
    /// models, and it is compiled in — so there is nothing to page through and
    /// no cursor to keep. `next_page_token` is returned empty, which is the
    /// honest answer rather than a paging scheme with one page.
    pub(super) fn list_catalog(
        &self,
        request: Request<ListCatalogModelsRequest>,
    ) -> Result<Response<ListCatalogModelsResponse>, Status> {
        let filter = request.into_inner().filter;
        let wanted = filter
            .split_once('=')
            .filter(|(key, _)| key.trim() == "modality")
            .map(|(_, value)| value.trim().trim_matches('"').to_owned());

        // Read once for the whole listing rather than per entry: it takes a
        // lock, and the answer cannot change between two rows of one response.
        let resident = self.embeddings.model_name();
        let catalog_models = self
            .catalog
            .entries()
            .iter()
            .filter(|e| wanted.as_deref().is_none_or(|m| e.modality.as_str() == m))
            .map(|e| convert::entry(e, &self.store, resident.as_deref()))
            .collect();

        Ok(Response::new(ListCatalogModelsResponse {
            catalog_models,
            next_page_token: String::new(),
        }))
    }

    /// One catalog model by name.
    pub(super) fn get_catalog(
        &self,
        request: Request<GetCatalogModelRequest>,
    ) -> Result<Response<CatalogModel>, Status> {
        let name = request.into_inner().name;
        let entry = self
            .catalog
            .get(catalog_id(&name))
            .ok_or_else(|| Status::not_found(format!("no catalog model called {name:?}")))?;
        Ok(Response::new(convert::entry(
            entry,
            &self.store,
            self.embeddings.model_name().as_deref(),
        )))
    }
}
