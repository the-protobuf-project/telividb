//! One collection's points: writing, reading, and searching them.

use crate::convert;
use crate::error::Result;
use crate::names;
use crate::search::SearchResults;
use telividb_proto::point::v1 as wire;
use telividb_proto::point::v1::points_client::PointsClient;
use tonic::transport::Channel;

/// A handle to one collection's points.
///
/// Holds the collection's id so no method takes it again, which is the
/// repetition that makes the generated client tedious to use directly — every
/// request there carries either a parent or a full resource name.
#[derive(Clone)]
pub struct Collection {
    points: PointsClient<Channel>,
    id: String,
}

impl std::fmt::Debug for Collection {
    /// Names the collection, which is the only part worth seeing.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Collection({})", self.id)
    }
}

impl Collection {
    /// Bind a handle to `id` over an existing connection.
    pub(crate) fn new(points: PointsClient<Channel>, id: String) -> Self {
        Self { points, id }
    }

    /// The collection's id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The underlying points client, for the read half in
    /// `collection_read.rs`.
    pub(crate) fn points_mut(&mut self) -> &mut PointsClient<Channel> {
        &mut self.points
    }

    /// Store a vector under `point_id` in the named field.
    pub async fn insert(&mut self, point_id: &str, field: &str, vector: &[f32]) -> Result<String> {
        self.create(point_id, convert::point_with_vector(field, vector))
            .await
    }

    /// Store a vector together with the text it was computed from.
    ///
    /// The text travels as `content_ref.inline_text`, so a later search can
    /// hand it straight back — without it, a result is a bare id the caller
    /// has to resolve against their own storage before it means anything.
    ///
    /// Inline suits short text. Anything large belongs outside the database
    /// behind a URI (invariant 19), which is what the rest of `ContentRef`
    /// describes.
    pub async fn insert_with_text(
        &mut self,
        point_id: &str,
        field: &str,
        vector: &[f32],
        text: &str,
    ) -> Result<String> {
        let mut point = convert::point_with_vector(field, vector);
        point.content_ref = Some(wire::ContentRef {
            uri: String::new(),
            range_start: 0,
            range_end: 0,
            sha256: Default::default(),
            inline_text: text.to_owned(),
        });
        self.create(point_id, point).await
    }

    /// Store many points.
    ///
    /// Each entry is `(point_id, vector, text)`; an empty `text` stores no
    /// content reference. All of them share one named field.
    ///
    /// **Currently one request per point, not one for the batch.** The
    /// server's `BatchCreatePoints` returns `Unimplemented`, so batching here
    /// would fail rather than be slow. Said plainly instead of hidden because
    /// the difference is `n` round trips versus one, which is the whole reason
    /// a caller would reach for this — when the RPC lands, this becomes a
    /// single call with no change to the signature.
    ///
    /// Stops at the first failure and returns it, rather than continuing and
    /// reporting a partial write as success. Bulk ingest that tolerates bad
    /// records is a *job* with a reject file (invariant 11), not this.
    pub async fn insert_many(
        &mut self,
        field: &str,
        entries: &[(String, Vec<f32>, String)],
    ) -> Result<Vec<String>> {
        let mut created = Vec::with_capacity(entries.len());
        for (id, vector, text) in entries {
            let name = match text.is_empty() {
                true => self.insert(id, field, vector).await?,
                false => self.insert_with_text(id, field, vector, text).await?,
            };
            created.push(name);
        }
        Ok(created)
    }

    /// Search the named field for the `k` nearest points.
    ///
    /// Returns [`SearchResults`] rather than a bare `Vec`, so a partial answer
    /// stays distinguishable from an empty one — see that type for why.
    pub async fn search(&mut self, field: &str, query: &[f32], k: usize) -> Result<SearchResults> {
        let response = self
            .points
            .search_points(wire::SearchPointsRequest {
                parent: names::collection(&self.id),
                field_id: field.to_owned(),
                query: Some(convert::to_wire(query)),
                query_text: String::new(),
                page_size: k as i32,
                page_token: String::new(),
                candidate_breadth: 0,
                read_mask: None,
            })
            .await?
            .into_inner();

        Ok(SearchResults::from_wire(response))
    }

    /// Delete a point.
    pub async fn delete(&mut self, point_id: &str) -> Result<()> {
        self.points
            .delete_point(wire::DeletePointRequest {
                name: names::point(&self.id, point_id),
            })
            .await?;
        Ok(())
    }

    /// Send one create request.
    fn create(
        &mut self,
        point_id: &str,
        point: wire::Point,
    ) -> impl std::future::Future<Output = Result<String>> + '_ {
        let request = wire::CreatePointRequest {
            parent: names::collection(&self.id),
            point_id: point_id.to_owned(),
            point: Some(point),
        };
        async move {
            let created = self.points.create_point(request).await?.into_inner();
            Ok(names::id_of(&created.name).to_owned())
        }
    }
}
