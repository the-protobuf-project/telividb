//! What a search gives back.

use crate::convert;
use crate::names;
use telividb_proto::point::v1 as wire;

/// One matching point.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// The point's id within its collection, ready to pass back to
    /// [`Collection::get`] or [`Collection::delete`].
    ///
    /// The id rather than the full resource name, because that is what every
    /// other method on the handle takes — returning a full name here would
    /// make a search result the one thing a caller has to reformat before
    /// using.
    ///
    /// [`Collection::get`]: crate::Collection::get
    /// [`Collection::delete`]: crate::Collection::delete
    pub name: String,

    /// Similarity to the query, as the field's metric defines it.
    ///
    /// Higher is nearer for dot and cosine; lower is nearer for L2. The metric
    /// is a property of the field, so the SDK does not reorder or invert it —
    /// results arrive in the server's ranking, which is the authoritative one.
    pub score: f32,

    /// Text the point carries inline, when it carries any.
    ///
    /// `None` is ordinary: a point may reference content stored elsewhere
    /// (invariant 19), in which case there is nothing inline to return.
    pub text: Option<String>,
}

/// The outcome of a search, including whether it saw everything.
///
/// A plain `Vec<Hit>` would be the wrong return type. A search that could not
/// reach part of the corpus — a locked vault, an unavailable shard — still
/// returns hits, and a caller handed only those cannot tell "nothing matched"
/// from "nothing you can currently see matched". Rules 27 and 49 require that
/// distinction to survive, so it is carried here rather than dropped.
#[derive(Debug, Clone)]
pub struct SearchResults {
    hits: Vec<Hit>,
    complete: bool,
    locked_vaults: Vec<String>,
    answered: i32,
    total: i32,
}

impl SearchResults {
    /// Read a response off the wire.
    pub(crate) fn from_wire(response: wire::SearchPointsResponse) -> Self {
        let hits = response
            .results
            .into_iter()
            .filter_map(|result| {
                let point = result.point?;
                Some(Hit {
                    name: names::id_of(&point.name).to_owned(),
                    score: result.score,
                    text: convert::inline_text(&point),
                })
            })
            .collect();

        Self {
            hits,
            complete: response.complete,
            locked_vaults: response.locked_vaults,
            answered: response.answered_source_count,
            total: response.total_source_count,
        }
    }

    /// The matches, best first.
    pub fn hits(&self) -> &[Hit] {
        &self.hits
    }

    /// Take ownership of the matches.
    pub fn into_hits(self) -> Vec<Hit> {
        self.hits
    }

    /// How many matches came back.
    pub fn len(&self) -> usize {
        self.hits.len()
    }

    /// Whether nothing matched.
    ///
    /// Note this says nothing about *why*. Pair it with [`Self::is_complete`]
    /// before reporting "no results" to a user.
    pub fn is_empty(&self) -> bool {
        self.hits.is_empty()
    }

    /// Whether every source answered.
    ///
    /// `false` means the result is a partial view: something the caller might
    /// have been allowed to see was not searched. Single-node with nothing
    /// locked always reports `true`.
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Vaults that were locked, and so not searched.
    ///
    /// Named rather than merely counted, so a caller can tell the user which
    /// key would widen the result.
    pub fn locked_vaults(&self) -> &[String] {
        &self.locked_vaults
    }

    /// Sources that answered, out of the total asked.
    pub fn coverage(&self) -> (i32, i32) {
        (self.answered, self.total)
    }
}

impl<'a> IntoIterator for &'a SearchResults {
    type Item = &'a Hit;
    type IntoIter = std::slice::Iter<'a, Hit>;

    /// Iterate the hits, so `for hit in &results` reads naturally.
    fn into_iter(self) -> Self::IntoIter {
        self.hits.iter()
    }
}

#[cfg(test)]
#[path = "search_test.rs"]
mod tests;
