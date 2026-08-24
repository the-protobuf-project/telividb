//! The Collections service.
//!
//! The catalogue is what makes a collection a *declaration* rather than a
//! side effect of the first write. See [`telividb_core::Collection`] for why
//! that matters: a field's width, metric and model are bound here, so a later
//! point that disagrees is refused instead of quietly widening the field.

use super::collection_convert::{to_domain, to_wire};
use crate::error::storage_status;
use std::path::PathBuf;
use std::sync::Arc;
use telividb_core::ResourceName;
use telividb_proto::collection::v1::collections_server::Collections;
use telividb_proto::collection::v1::{
    Collection, CreateCollectionRequest, DeleteCollectionRequest, GetCollectionRequest,
    ListCollectionsRequest, ListCollectionsResponse,
};
use telividb_storage::RedbCollectionStore;
use telividb_telemetry::{fields, logger, redact};
use tonic::{Request, Response, Status};

/// Handles collection create, read, list and delete.
pub struct CollectionSvc {
    catalogue: Arc<RedbCollectionStore>,
    data_dir: PathBuf,
}

impl CollectionSvc {
    /// Open the catalogue beneath `data_dir`.
    ///
    /// Opened once, at construction, rather than per request: `redb` takes an
    /// exclusive file lock, so two concurrent opens of the same file fail.
    pub fn open(data_dir: PathBuf) -> Result<Self, telividb_storage::Error> {
        let catalogue = RedbCollectionStore::open(&data_dir.join("collections.redb"))?;
        Ok(Self {
            catalogue: Arc::new(catalogue),
            data_dir,
        })
    }

    /// The catalogue, shared with the point service so both agree about which
    /// collections exist.
    pub fn catalogue(&self) -> Arc<RedbCollectionStore> {
        Arc::clone(&self.catalogue)
    }
}

/// Parse a resource name, mapping a bad one to `InvalidArgument`.
fn parse_name(raw: &str) -> Result<ResourceName, Status> {
    ResourceName::parse(raw).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl Collections for CollectionSvc {
    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<Collection>, Status> {
        let req = request.into_inner();

        // Collection names reach telemetry, so a vault name would leak simply
        // by being created.
        logger::info!("create collection").with_data(&serde_json::json!({
            fields::COLLECTION: redact::collection_label(&req.collection_id),
        }));

        if req.collection_id.is_empty() {
            return Err(Status::invalid_argument(
                "collection_id must not be empty: it forms the final path segment \
                 of the collection's resource name",
            ));
        }
        let name = parse_name(&format!("collections/{}", req.collection_id))?;
        let payload = req.collection.unwrap_or_default();

        if payload.descriptor_set.is_empty() {
            return Err(Status::invalid_argument(
                "descriptor_set is required: the engine never parses .proto, \
                 it consumes a compiled FileDescriptorSet",
            ));
        }

        // The fingerprint every segment written under this schema will carry.
        let fingerprint = telividb_core::Fingerprint::of(&payload.descriptor_set);
        let collection = to_domain(name.clone(), fingerprint, &payload)?;

        let catalogue = Arc::clone(&self.catalogue);
        let descriptor_set = payload.descriptor_set.to_vec();
        let stored = descriptor_set.clone();
        let created = tokio::task::spawn_blocking(move || {
            catalogue.create(&collection, &descriptor_set).map(|ok| (ok, collection))
        })
        .await
        .map_err(|e| Status::internal(format!("catalogue task failed: {e}")))?
        .map_err(|e| storage_status(&e))?;

        match created {
            (true, collection) => Ok(Response::new(to_wire(&collection, stored))),
            (false, _) => Err(Status::already_exists(format!(
                "collection {} already exists",
                name.as_str()
            ))),
        }
    }

    async fn get_collection(
        &self,
        request: Request<GetCollectionRequest>,
    ) -> Result<Response<Collection>, Status> {
        let name = parse_name(&request.into_inner().name)?;
        logger::debug!("get collection").with_data(&serde_json::json!({
            fields::COLLECTION: redact::collection_label(name.as_str()),
        }));

        let catalogue = Arc::clone(&self.catalogue);
        let found = tokio::task::spawn_blocking(move || catalogue.entry(&name))
            .await
            .map_err(|e| Status::internal(format!("catalogue task failed: {e}")))?
            .map_err(|e| storage_status(&e))?;

        match found {
            Some((collection, descriptor_set)) => {
                Ok(Response::new(to_wire(&collection, descriptor_set)))
            }
            None => Err(Status::not_found("collection not found")),
        }
    }

    async fn list_collections(
        &self,
        _request: Request<ListCollectionsRequest>,
    ) -> Result<Response<ListCollectionsResponse>, Status> {
        // Deliberately silent. `ListCollections` always succeeds, which makes
        // it the natural liveness probe, and a polling client would turn one
        // record per call into unbounded console volume.
        let catalogue = Arc::clone(&self.catalogue);
        let found = tokio::task::spawn_blocking(move || catalogue.list())
            .await
            .map_err(|e| Status::internal(format!("catalogue task failed: {e}")))?
            .map_err(|e| storage_status(&e))?;

        Ok(Response::new(ListCollectionsResponse {
            // The descriptor set is omitted from a list: it is the largest
            // part of a record by far, and a caller listing names does not
            // want megabytes of schema per entry. `GetCollection` carries it.
            collections: found.iter().map(|c| to_wire(c, Vec::new())).collect(),
            next_page_token: String::new(),
        }))
    }

    async fn delete_collection(
        &self,
        request: Request<DeleteCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        let name = parse_name(&request.into_inner().name)?;
        logger::info!("delete collection").with_data(&serde_json::json!({
            fields::COLLECTION: redact::collection_label(name.as_str()),
        }));

        let catalogue = Arc::clone(&self.catalogue);
        let dir = self.data_dir.join(name.leaf());
        let existed = tokio::task::spawn_blocking(move || {
            let existed = catalogue.delete(&name)?;
            if existed && dir.exists() {
                // The catalogue entry goes first: a crash between the two
                // leaves data with no entry, which reads as absent. The other
                // order would leave an entry pointing at nothing.
                std::fs::remove_dir_all(&dir)?;
            }
            Ok::<_, telividb_storage::Error>(existed)
        })
        .await
        .map_err(|e| Status::internal(format!("catalogue task failed: {e}")))?
        .map_err(|e| storage_status(&e))?;

        match existed {
            true => Ok(Response::new(())),
            false => Err(Status::not_found("collection not found")),
        }
    }
}
