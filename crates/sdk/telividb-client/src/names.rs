//! Building resource names, in one place.
//!
//! Every request addresses either a resource or its parent, and the two
//! spellings must agree exactly — `collections/{c}` for a collection and
//! `collections/{c}/points/{p}` for a point beneath it (AIP-122). Assembled at
//! each call site with `format!`, they eventually diverge: one gains a
//! trailing slash, another omits the collection prefix, and the resulting
//! `NotFound` points at the caller rather than at the typo.
//!
//! Callers of this crate pass bare ids — `"documents"`, `"doc-1"` — and never
//! see these strings at all.

/// The collection segment of every name.
const COLLECTIONS: &str = "collections";

/// The points segment beneath a collection.
const POINTS: &str = "points";

/// The parent every point request is addressed under.
///
/// Also the name a collection is fetched and deleted by, so the two spellings
/// cannot drift apart.
pub fn collection(collection: &str) -> String {
    format!("{COLLECTIONS}/{collection}")
}

/// One point's full name, nested under its collection.
///
/// Built from [`collection`]'s output so a point is always addressed beneath
/// the same parent a create sent it to.
pub fn point(collection: &str, point: &str) -> String {
    format!("{COLLECTIONS}/{collection}/{POINTS}/{point}")
}

/// The trailing id of a resource name, which is what a caller passes back in.
///
/// Returns the whole string when there is no `/`, so a name that is already an
/// id survives a round trip unchanged rather than becoming empty.
pub fn id_of(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

#[cfg(test)]
#[path = "names_test.rs"]
mod tests;
