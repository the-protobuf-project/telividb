//! Generated gRPC types and service stubs.
//!
//! # Generated, not built
//!
//! Everything under `generated/` is produced by `buf` from `proto/` and
//! **committed to the repository**. There is no build script, so `cargo build`
//! needs no protobuf toolchain at all — the same rule that keeps C compilers
//! out of the default build.
//!
//! It also means a renumbered field or a renamed service appears in a pull
//! request diff rather than materialising silently at the next build.
//!
//! Regenerate with `cargo xtask gen-proto`. CI runs the same command and fails
//! if the result differs from what is committed, so the two cannot drift.
//!
//! **Never edit `generated/` by hand.** Change the `.proto` and regenerate.
//!
//! # Why one generated file, and how the structure is recovered
//!
//! prost maps a protobuf **package** onto a Rust module, not a file onto a
//! file. Every `.proto` in `episteme.v1` therefore generates into one
//! `episteme.v1.rs`, and no plugin option changes that — it is protobuf
//! semantics rather than a missing flag. Verified in both directions: two files
//! in one package produce one output, and splitting the package produces two.
//!
//! Splitting the package is the wrong trade. buf requires a package to live in
//! a directory matching its name *and* to carry its own version suffix, so
//! `search.proto` would become package `episteme.v1.search.v1` and every type
//! reference would grow to match. Google's own APIs keep many files in one
//! versioned package for exactly this reason.
//!
//! So the proto layout stays flat and the *logical* structure is recovered
//! here: [`collection`], [`point`], [`search`] and [`types`] re-export the
//! subset each `.proto` defines. Callers get
//! `episteme_proto::search::SearchRequest` without the package name being
//! contorted to produce it.
//!
//! Licensed Apache-2.0 rather than AGPL, so a proprietary application can talk
//! to an episteme server without a commercial licence.
#![allow(missing_docs, clippy::all, rustdoc::all)]

/// The `episteme.v1` API surface, exactly as generated.
pub mod v1 {
    include!("generated/episteme.v1.rs");
    include!("generated/episteme.v1.tonic.rs");
}

/// Shared value types, from `common.proto`.
pub mod types {
    pub use super::v1::{Codec, ContentRef, IndexKind, Metric, Span, Vector};
}

/// Collection lifecycle, from `collection.proto`.
pub mod collection {
    pub use super::v1::collection_service_client::CollectionServiceClient;
    pub use super::v1::collection_service_server::{CollectionService, CollectionServiceServer};
    pub use super::v1::{
        CreateCollectionRequest, CreateCollectionResponse, DescribeCollectionRequest,
        DescribeCollectionResponse, DropCollectionRequest, DropCollectionResponse,
        ListCollectionsRequest, ListCollectionsResponse, VectorFieldSpec,
    };
}

/// Point lifecycle, from `point.proto`.
pub mod point {
    pub use super::v1::point_service_client::PointServiceClient;
    pub use super::v1::point_service_server::{PointService, PointServiceServer};
    pub use super::v1::{
        DeleteRequest, DeleteResponse, GetRequest, GetResponse, NamedVector, Point, Rejection,
        UpsertRequest, UpsertResponse,
    };
}

/// Vector retrieval, from `search.proto`.
pub mod search {
    pub use super::v1::search_service_client::SearchServiceClient;
    pub use super::v1::search_service_server::{SearchService, SearchServiceServer};
    pub use super::v1::{
        BatchSearchRequest, BatchSearchResponse, Hit, SearchRequest, SearchResponse, SearchStats,
    };
}

/// Serialized `FileDescriptorSet` for every service in this crate.
///
/// Served over gRPC reflection so `grpcurl`, generic clients and MCP bridges can
/// introspect the API without being shipped the protos first.
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/descriptor.bin");

pub use v1::*;
