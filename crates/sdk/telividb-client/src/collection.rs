//! One collection's points: writing, reading, and searching them.

use crate::convert;
use crate::error::Result;
use crate::names;
use crate::search::SearchResults;
use telividb_buffers::protobuf::point::v1 as wire;
use telividb_buffers::protobuf::point::v1::points_client::PointsClient;
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
    pub async fn insert_with_text(
        &mut self,
        point_id: &str,
        field: &str,
        vector: &[f32],
        text: &str,
    ) -> Result<String> {
        let mut point = convert::point_with_vector(field, vector);
        point.content_ref = Some(convert::inline_ref(text));
        self.create(point_id, point).await
    }

    /// Store many points in one request.
    ///
    /// Each entry is `(point_id, vector, text)`; an empty `text` stores no
    /// content reference. All of them share one named field.
    ///
    /// One round trip rather than `n`, which is the reason to reach for this
    /// over a loop of [`insert`](Self::insert). It is not faster on the server
    /// — each point still takes the same path through the WAL and `redb` — so
    /// what it saves is the request/response pair per row, which for an import
    /// of any size is the whole cost.
    ///
    /// Stops at the first failure and returns it, rather than continuing and
    /// reporting a partial write as success. The server names which entry
    /// failed and how many were written before it, so a caller can fix that
    /// row and resubmit from there. Note the write is *not* atomic: points
    /// before the failure stay written. Bulk ingest that tolerates bad records
    /// is a *job* with a reject file (invariant 11), not this.
    ///
    /// An empty `entries` is a no-op returning no names, rather than an error.
    /// The server refuses an empty batch — a request with nothing in it is a
    /// mistake — but a caller looping over a possibly-empty slice is not making
    /// that mistake, so the check belongs here rather than as a round trip that
    /// can only fail.
    pub async fn insert_many(
        &mut self,
        field: &str,
        entries: &[(String, Vec<f32>, String)],
    ) -> Result<Vec<String>> {
        if entries.is_empty() {
            return Ok(Vec::new());
        }

        let parent = names::collection(&self.id);
        let requests = entries
            .iter()
            .map(|(id, vector, text)| {
                let mut point = convert::point_with_vector(field, vector);
                if !text.is_empty() {
                    point.content_ref = Some(convert::inline_ref(text));
                }
                wire::CreatePointRequest {
                    // Left empty: the batch's `parent` is authoritative, and
                    // the server refuses an item that disagrees with it.
                    parent: String::new(),
                    point_id: id.clone(),
                    point: Some(point),
                }
            })
            .collect();

        let created = self
            .points
            .batch_create_points(wire::BatchCreatePointsRequest { parent, requests })
            .await?
            .into_inner();

        Ok(created
            .points
            .iter()
            .map(|p| names::id_of(&p.name).to_owned())
            .collect())
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
