//! The Rust SDK for telividb — a gRPC client.
//!
//! ```no_run
//! # async fn example() -> telividb_client::Result<()> {
//! use telividb_client::Client;
//!
//! let mut db = Client::connect("http://127.0.0.1:7700").await?;
//! let mut docs = db.collection("documents");
//!
//! docs.insert("doc-1", "text", &[0.1, 0.2, 0.3]).await?;
//!
//! let found = docs.search("text", &[0.1, 0.2, 0.3], 5).await?;
//! for hit in found.hits() {
//!     println!("{:.4}  {}", hit.score, hit.name);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # What this hides, and why each one matters
//!
//! **Resource names.** A point is addressed as
//! `collections/{collection}/points/{point}`, and every request takes either
//! that or its parent. Built by hand at each call site, the two eventually
//! disagree — so they are built in exactly one place here.
//!
//! **Vector encoding.** Vectors cross the wire as `bytes` holding raw
//! little-endian `f32`, never `repeated float`: protobuf encodes a repeated
//! scalar element by element, which is 768 varint operations per message on
//! the hot path. Callers pass `&[f32]` and never see the conversion.
//!
//! **Incompleteness.** A search that could not see everything says so, and
//! [`SearchResults`] carries that rather than handing back a bare `Vec`. A
//! caller must be able to tell "no results" from "no results you can currently
//! see" (CLAUDE.md rules 27 and 49).
//!
//! # What it deliberately does not do
//!
//! It does not embed text. Inference is server-side by design — one inference
//! server for ingest, query encoding and plugin compute alike (rules 42–45) —
//! so a client that loaded its own model would be a second, unpoliced path to
//! the same vectors. Today that means the caller supplies vectors; see the
//! crate README for the proto work that would let text be sent directly.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod client;
mod collection;
mod collection_read;
mod collection_text;
mod convert;
mod error;
mod models;
mod names;
mod new_collection;
mod page;
mod record;
mod search;
mod system;
mod tenancy;
mod tenancy_types;

pub use client::Client;
pub use system::{BudgetSource, System};
pub use tenancy_types::{Organization, Project, Protection, Space};

pub use collection::Collection;
pub use error::{Error, Result};
pub use new_collection::{Metric, NewCollection};
pub use record::Record;
pub use search::{Hit, SearchResults};
/// The generated wire types.
///
/// Re-exported because several methods return them directly — a caller that
/// receives a `CatalogModel` has to be able to name one, and requiring it to
/// depend on `telividb-buffers` separately would let the two drift to different
/// versions of the same schema.
pub use telividb_buffers::protobuf as wire;
