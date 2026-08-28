//! Reading points back.
//!
//! Split from `collection.rs` so that file stays about writing and searching.
//! The two halves share only the handle.

use crate::collection::Collection;
use crate::error::{Error, Result};
use crate::names;
use crate::record::Record;
use telividb_buffers::protobuf::point::v1 as wire;

impl Collection {
    /// Fetch one point.
    ///
    /// `None` rather than an error when it does not exist: asking for
    /// something not yet written is an ordinary outcome, and forcing every
    /// caller to match on a `NotFound` variant to express "absent" makes the
    /// common path the noisy one.
    pub async fn get(&mut self, point_id: &str) -> Result<Option<Record>> {
        let request = wire::GetPointRequest {
            name: names::point(self.id(), point_id),
            read_mask: None,
        };

        match self.points_mut().get_point(request).await {
            Ok(response) => Ok(Some(Record::from_wire(response.into_inner())?)),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(status) => Err(Error::from(status)),
        }
    }

    /// Every point in the collection.
    ///
    /// Pages are followed to the end. That is right for inspection and for
    /// small collections, and wrong for a large one — a corpus of any size
    /// should be exported as a bulk job (invariant 10) rather than pulled
    /// through here into memory.
    pub async fn list(&mut self) -> Result<Vec<Record>> {
        let parent = names::collection(self.id());
        let mut records = Vec::new();
        let mut page_token = String::new();

        loop {
            let response = self
                .points_mut()
                .list_points(wire::ListPointsRequest {
                    parent: parent.clone(),
                    page_size: 0,
                    page_token: page_token.clone(),
                    read_mask: None,
                })
                .await?
                .into_inner();

            for point in response.points {
                records.push(Record::from_wire(point)?);
            }

            // An empty token ends the sequence (AIP-158).
            if response.next_page_token.is_empty() {
                return Ok(records);
            }
            page_token = response.next_page_token;
        }
    }
}
