//! Generated gRPC types and service stubs.
//!
//! # Generated, not built
//!
//! Everything under `generated/` is produced by `buf` from `protobuf/` and
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
//! # One package per resource, each self-contained
//!
//! Each resource owns a versioned package — `telividb.collection.v1`,
//! `telividb.point.v1` — and declares every type it uses. Within a package,
//! files are split by role: the resource, its messages, its search types, and
//! its service.
//!
//! There is deliberately **no shared package**. A package reaching into one
//! would couple its versioning to that package's, which is what AIP-215
//! forbids and what the linter flags. The cost is that `Vector` and `Span` are
//! declared where they are used; the benefit is that a consumer of
//! `telividb.point.v1` needs nothing else.
//!
//! This is also what buf requires (a package must live in a matching
//! directory) and what prost rewards (a package becomes a module, so each
//! generates its own file).
#![allow(missing_docs, clippy::all, rustdoc::all)]

/// The Collection resource and its service.
///
/// Self-contained: it declares every type it uses. A package that reached into
/// a shared one would couple its versioning to that package's, which is what
/// AIP-215 exists to prevent.
pub mod collection {
    /// Version 1 of the Collection resource and service.
    pub mod v1 {
        include!("generated/telividb/collection/v1/telividb.collection.v1.rs");
        include!("generated/telividb/collection/v1/telividb.collection.v1.tonic.rs");
    }
}

/// The Point resource, its service, and search.
///
/// Search lives here rather than in a service of its own, because searching is
/// an operation over points — `points:search` under a collection, not a
/// parallel hierarchy.
pub mod point {
    /// Version 1 of the Point resource, service and search.
    pub mod v1 {
        include!("generated/telividb/point/v1/telividb.point.v1.rs");
        include!("generated/telividb/point/v1/telividb.point.v1.tonic.rs");
    }
}

/// Serialized `FileDescriptorSet` for every service in this crate.
///
/// Served over gRPC reflection so `grpcurl`, generic clients and MCP bridges can
/// introspect the API without being shipped the protos first.
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/descriptor.bin");
