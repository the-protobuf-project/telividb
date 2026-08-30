//! The connection, and collection-level operations.

use crate::collection::Collection;
use crate::error::Result;
use crate::names;
use crate::new_collection::NewCollection;
use telividb_buffers::protobuf::collection::v1 as wire;
use telividb_buffers::protobuf::collection::v1::collections_client::CollectionsClient;
use telividb_buffers::protobuf::models::v1::models_client::ModelsClient;
use telividb_buffers::protobuf::point::v1::points_client::PointsClient;
use telividb_buffers::protobuf::tenancy::v1::organizations_client::OrganizationsClient;
use telividb_buffers::protobuf::tenancy::v1::projects_client::ProjectsClient;
use telividb_buffers::protobuf::tenancy::v1::spaces_client::SpacesClient;
use tonic::transport::Channel;

/// A connection to a telividb server.
///
/// Cheap to clone in effect: both generated clients share one HTTP/2 channel,
/// which multiplexes concurrent requests. Opening a second `Client` to the
/// same server opens a second connection for no benefit.
#[derive(Clone)]
pub struct Client {
    collections: CollectionsClient<Channel>,
    points: PointsClient<Channel>,
    /// The model catalog. See `models.rs` for what it offers.
    pub(crate) models: ModelsClient<Channel>,
    /// Tenancy. See `tenancy.rs` — all three share the one channel above.
    pub(crate) organizations: OrganizationsClient<Channel>,
    pub(crate) projects: ProjectsClient<Channel>,
    pub(crate) spaces: SpacesClient<Channel>,
}

impl std::fmt::Debug for Client {
    /// Written by hand because the generated clients are not `Debug`, and
    /// because their internals — channel state, interceptors — are noise to
    /// anyone debugging their own call.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Client { connected }")
    }
}

impl Client {
    /// Connect to a server, e.g. `http://127.0.0.1:7700`.
    ///
    /// The scheme is required — `tonic` needs it to decide between plaintext
    /// and TLS, and a bare `host:port` fails with a URI error that says
    /// nothing about the missing scheme.
    pub async fn connect(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();

        // Checked here rather than left to the transport. `Channel` accepts a
        // schemeless string and fails much later, during `connect`, as a
        // generic transport error — so the one thing the caller needs to know
        // arrives buried in a message about a connection that was never
        // attempted.
        if !endpoint.contains("://") {
            return Err(crate::Error::InvalidArgument {
                message: format!(
                    "endpoint {endpoint:?} has no scheme; it needs one, \
                     e.g. http://127.0.0.1:7700"
                ),
            });
        }

        let channel = Channel::from_shared(endpoint)
            .map_err(|e| crate::Error::InvalidArgument {
                message: format!("not a valid endpoint: {e}"),
            })?
            .connect()
            .await?;

        Ok(Self {
            collections: CollectionsClient::new(channel.clone()),
            points: PointsClient::new(channel.clone()),
            models: ModelsClient::new(channel.clone()),
            organizations: OrganizationsClient::new(channel.clone()),
            projects: ProjectsClient::new(channel.clone()),
            spaces: SpacesClient::new(channel),
        })
    }

    /// A handle to one collection's points.
    ///
    /// Does not check that the collection exists, and makes no call: it is a
    /// name plus a shared channel. A handle to something absent fails on first
    /// use with `NotFound`, which is where the caller can act on it anyway.
    pub fn collection(&self, id: impl Into<String>) -> Collection {
        Collection::new(self.points.clone(), id.into())
    }

    /// Create a collection, declaring the vector fields its points will
    /// carry.
    ///
    /// ```no_run
    /// # async fn example(db: &mut telividb_client::Client) -> telividb_client::Result<()> {
    /// use telividb_client::NewCollection;
    ///
    /// db.create_collection(
    ///     NewCollection::new("documents", descriptor_set())
    ///         .text_field("text", 768),
    /// )
    /// .await?;
    /// # Ok(()) }
    /// # fn descriptor_set() -> Vec<u8> { Vec::new() }
    /// ```
    ///
    /// Points cannot be written to a collection that does not exist — the
    /// server refuses rather than creating one implicitly, which is what makes
    /// the declaration above worth anything.
    pub async fn create_collection(&mut self, spec: NewCollection) -> Result<String> {
        let created = self
            .collections
            .create_collection(wire::CreateCollectionRequest {
                collection_id: spec.id.clone(),
                collection: Some(spec.to_wire()),
            })
            .await?
            .into_inner();

        Ok(names::id_of(&created.name).to_owned())
    }

    /// Every collection's id.
    ///
    /// Pages are followed to the end, so a caller gets the whole list rather
    /// than a first page they might mistake for it. The count is bounded by
    /// how many collections exist, which is a number an operator chose.
    pub async fn list_collections(&mut self) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        let mut page_token = String::new();

        loop {
            let response = self
                .collections
                .list_collections(wire::ListCollectionsRequest {
                    page_size: 0,
                    page_token: page_token.clone(),
                })
                .await?
                .into_inner();

            ids.extend(
                response
                    .collections
                    .iter()
                    .map(|c| names::id_of(&c.name).to_owned()),
            );

            // An empty token ends the sequence (AIP-158). Comparing against
            // the previous token as well would loop forever on a server that
            // wrongly repeats one, so the emptiness check is the only exit.
            if response.next_page_token.is_empty() {
                return Ok(ids);
            }
            page_token = response.next_page_token;
        }
    }

    /// Delete a collection and everything in it.
    pub async fn delete_collection(&mut self, id: &str) -> Result<()> {
        self.collections
            .delete_collection(wire::DeleteCollectionRequest {
                name: names::collection(id),
            })
            .await?;
        Ok(())
    }
}
