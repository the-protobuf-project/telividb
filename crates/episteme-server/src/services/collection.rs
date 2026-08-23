//! Collection lifecycle.

use episteme_proto::v1::collection_service_server::CollectionService;
use episteme_proto::v1::{
    CreateCollectionRequest, CreateCollectionResponse, DescribeCollectionRequest,
    DescribeCollectionResponse, DropCollectionRequest, DropCollectionResponse,
    ListCollectionsRequest, ListCollectionsResponse,
};
use episteme_telemetry::{fields, redact};
use tonic::{Request, Response, Status};

/// Collection lifecycle service.
///
/// Currently a wiring shell: it validates arguments and computes fingerprints,
/// but has no catalogue behind it. That arrives with the schema adapter, which
/// is blocked on the `episteme.v1` vocabulary.
#[derive(Debug, Default)]
pub struct CollectionSvc {}

#[tonic::async_trait]
impl CollectionService for CollectionSvc {
    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<CreateCollectionResponse>, Status> {
        let req = request.into_inner();

        // Collection names reach telemetry, so a vault name would leak simply
        // by being created.
        let span = tracing::info_span!(
            "episteme.collection.create",
            { fields::COLLECTION } = redact::collection_label(&req.name),
            fields = req.vector_fields.len(),
        );
        let _guard = span.enter();

        if req.name.is_empty() {
            return Err(Status::invalid_argument(
                "collection name must not be empty",
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

    async fn describe_collection(
        &self,
        _request: Request<DescribeCollectionRequest>,
    ) -> Result<Response<DescribeCollectionResponse>, Status> {
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
        Ok(Response::new(ListCollectionsResponse { names: Vec::new() }))
    }

    async fn drop_collection(
        &self,
        _request: Request<DropCollectionRequest>,
    ) -> Result<Response<DropCollectionResponse>, Status> {
        Err(Status::unimplemented(
            "collection catalogue is not yet implemented",
        ))
    }
}
