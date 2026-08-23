//! The Collections service.

use episteme_proto::collection::v1::collections_server::Collections;
use episteme_proto::collection::v1::{
    Collection, CreateCollectionRequest, DeleteCollectionRequest, GetCollectionRequest,
    ListCollectionsRequest, ListCollectionsResponse,
};
use episteme_telemetry::{fields, redact};
use tonic::{Request, Response, Status};

/// Handles collection create, read, list and delete.
///
/// Currently a wiring shell: it validates arguments and computes fingerprints,
/// but has no catalogue behind it. That arrives with the schema adapter, which
/// is blocked on the `episteme.v1` vocabulary.
#[derive(Debug, Default)]
pub struct CollectionSvc {}

#[tonic::async_trait]
impl Collections for CollectionSvc {
    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<Collection>, Status> {
        let req = request.into_inner();

        // Collection names reach telemetry, so a vault name would leak simply
        // by being created.
        let span = tracing::info_span!(
            "episteme.collection.create",
            { fields::COLLECTION } = redact::collection_label(&req.collection_id),
        );
        let _guard = span.enter();

        if req.collection_id.is_empty() {
            return Err(Status::invalid_argument(
                "collection_id must not be empty: it forms the final path segment \
                 of the collection's resource name",
            ));
        }
        if req.descriptor_set.is_empty() {
            return Err(Status::invalid_argument(
                "descriptor_set is required: the engine never parses .proto, \
                 it consumes a compiled FileDescriptorSet",
            ));
        }

        // The fingerprint every segment written under this schema will carry.
        let fingerprint = episteme_core::Fingerprint::of(&req.descriptor_set);
        tracing::info!(schema = %fingerprint, "collection schema fingerprinted");

        Err(Status::unimplemented(
            "collection catalogue is not yet implemented; \
             blocked on the episteme.v1 schema vocabulary",
        ))
    }

    async fn get_collection(
        &self,
        _request: Request<GetCollectionRequest>,
    ) -> Result<Response<Collection>, Status> {
        Err(Status::unimplemented(
            "collection catalogue is not yet implemented",
        ))
    }

    async fn list_collections(
        &self,
        _request: Request<ListCollectionsRequest>,
    ) -> Result<Response<ListCollectionsResponse>, Status> {
        // Answers rather than failing, so a client can confirm the server is
        // wired end to end before any collection exists.
        Ok(Response::new(ListCollectionsResponse {
            collections: Vec::new(),
            next_page_token: String::new(),
        }))
    }

    async fn delete_collection(
        &self,
        _request: Request<DeleteCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented(
            "collection catalogue is not yet implemented",
        ))
    }
}
