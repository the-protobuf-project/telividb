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
//! # One package per resource
//!
//! Each resource owns a versioned package — `episteme.collection.v1`,
//! `episteme.point.v1`, `episteme.search.v1` — with shared value types in
//! `episteme.shared.v1`. Within a package, files are split by role: the
//! resource, its request and response messages, and its service.
//!
//! This is what buf requires (a package must live in a matching directory) and
//! what prost rewards (a package becomes a module, so each generates its own
//! file). The module layout below mirrors it exactly.
#![allow(missing_docs, clippy::all, rustdoc::all)]

/// Value types shared across every resource.
pub mod shared {
    /// Version 1 of the shared value types.
    pub mod v1 {
        include!("generated/episteme/shared/v1/episteme.shared.v1.rs");
    }
}

/// The Collection resource and its service.
pub mod collection {
    /// Version 1 of the Collection resource and service.
    pub mod v1 {
        include!("generated/episteme/collection/v1/episteme.collection.v1.rs");
        include!("generated/episteme/collection/v1/episteme.collection.v1.tonic.rs");
    }
}

/// The Point resource and its service.
pub mod point {
    /// Version 1 of the Point resource and service.
    pub mod v1 {
        include!("generated/episteme/point/v1/episteme.point.v1.rs");
        include!("generated/episteme/point/v1/episteme.point.v1.tonic.rs");
    }
}

/// Vector retrieval: search requests, results and the Search service.
pub mod search {
    /// Version 1 of the search request and result types.
    pub mod v1 {
        include!("generated/episteme/search/v1/episteme.search.v1.rs");
        include!("generated/episteme/search/v1/episteme.search.v1.tonic.rs");
    }
}

/// Serialized `FileDescriptorSet` for every service in this crate.
///
/// Served over gRPC reflection so `grpcurl`, generic clients and MCP bridges can
/// introspect the API without being shipped the protos first.
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/descriptor.bin");
