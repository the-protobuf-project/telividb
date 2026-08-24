//! AIP-122 resource names.
//!
//! Resource names are the **API-facing identity**: `collections/finance/points/doc-123`.
//! They are self-describing, they extend cleanly (`projects/{p}/collections/…`
//! if multi-tenancy ever arrives), and — the reason they matter most here —
//! grants can be expressed as *patterns* over them, so authorization scope has
//! a native syntax: `collections/finance/points/*`.
//!
//! # Three levels of identity, deliberately
//!
//! | Level | Type | Scope | Where it appears |
//! |---|---|---|---|
//! | API | [`ResourceName`] | global, human-readable | requests, responses, archives, edges |
//! | Interned | [`crate::ExternalId`] | one collection | `ids.bin`, fixed width |
//! | Row | [`crate::Ordinal`] | one segment | inside an index, never escapes |
//!
//! The middle level is not redundant. Segment files are fixed-stride so that an
//! mmap'd region casts to a slice with no copy; a variable-length string per row
//! would destroy that. Names are interned to a `u64` in the collection metadata,
//! and only the `u64` reaches storage.
//!
//! **Archives carry names, not interned ids.** An interned id is assigned per
//! collection, so it is meaningless — and potentially colliding — in another.
//! The importer re-interns on the way in.

mod name;
mod template;

pub use name::ResourceName;
pub use template::Template;
