//! Storing and searching by text, with the server doing the embedding.
//!
//! This is the surface most callers want: text in, ranked text out, with no
//! model on the client at all.
//!
//! The server embeds rather than the client, and that is a guarantee rather
//! than a convenience. A field's vectors are bound to one model identity (rule
//! 12), and vectors from two different models merged into one index do not
//! fail — recall degrades and every neighbour returned stays plausible. A
//! client holding its own model would be exactly that second source. Keeping
//! inference server-side also leaves one place where the policy check at the
//! inference boundary (rule 44) can attach.
//!
//! A server started without `--model` refuses these with a message naming the
//! flag, rather than accepting the text and storing nothing.

use crate::collection::Collection;
use crate::convert;
use crate::error::Result;
use crate::names;
use crate::search::SearchResults;
use telividb_proto::point::v1 as wire;

impl Collection {
    /// Store text, letting the server compute the vector.
    ///
    /// The text is also stored as the point's inline content, so a later
    /// search hands it straight back — see [`crate::Hit::text`].
    pub async fn add_text(&mut self, point_id: &str, field: &str, text: &str) -> Result<String> {
        let request = wire::CreatePointRequest {
            parent: names::collection(self.id()),
            point_id: point_id.to_owned(),
            point: Some(convert::point_from_text(field, text)),
        };

        let created = self.points_mut().create_point(request).await?.into_inner();
        Ok(names::id_of(&created.name).to_owned())
    }

    /// Store many texts.
    ///
    /// One request per entry today, for the same reason [`Collection::insert_many`]
    /// is: the server's batch RPC is not implemented yet. Stops at the first
    /// failure rather than reporting a partial write as success.
    pub async fn add_texts(
        &mut self,
        field: &str,
        entries: &[(String, String)],
    ) -> Result<Vec<String>> {
        let mut created = Vec::with_capacity(entries.len());
        for (id, text) in entries {
            created.push(self.add_text(id, field, text).await?);
        }
        Ok(created)
    }

    /// Search with text, letting the server encode the query.
    ///
    /// Encoded as a *query* rather than as a document, which is not the same
    /// operation: asymmetric models prepend a different task prefix to each,
    /// and using the wrong one lowers recall while returning well-formed
    /// vectors. The server knows which, because the field declares its query
    /// encoder — for a joint model that is a different tower entirely
    /// (invariant 18), which a client could not express.
    pub async fn search_text(
        &mut self,
        field: &str,
        query: &str,
        k: usize,
    ) -> Result<SearchResults> {
        // Built before the mutable borrow: `self.id()` reads the handle, and
        // `points_mut()` holds it exclusively for the call.
        let request = wire::SearchPointsRequest {
            parent: names::collection(self.id()),
            field_id: field.to_owned(),
            query: None,
            query_text: query.to_owned(),
            page_size: k as i32,
            page_token: String::new(),
            candidate_breadth: 0,
            read_mask: None,
        };

        let response = self.points_mut().search_points(request).await?.into_inner();

        Ok(SearchResults::from_wire(response))
    }
}
