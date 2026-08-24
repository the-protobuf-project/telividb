//! Durability and on-disk layout.
//!
//! Three ideas carry this crate, and everything else follows from them:
//!
//! 1. **Sealed segments are immutable.** Once written, a segment's files are
//!    never rewritten. Mutation produces a new segment plus a tombstone bitmap.
//! 2. **The manifest is the only mutable pointer**, and it changes by atomic
//!    rename — so a set of segments becomes visible all at once or not at all.
//! 3. **Every structure is versioned**, with magic bytes, and an unknown
//!    version is refused rather than guessed at.
//!
//! Together these give lock-free reads, snapshot isolation for free, and — much
//! later — sharding, because a shard is just a set of segments.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod buffer;
pub mod compact;
pub mod error;
pub mod field;
pub mod format;
pub mod manifest;
pub mod meta;
pub mod ports;
pub mod segment;
pub mod tier;
pub mod wal;

pub use buffer::MutableBuffer;
pub use compact::{CompactionPlan, CompactionPolicy, CompactionResult, compact_field};
pub use error::{Error, Result};
pub use field::{DEFAULT_SEAL_BYTES, VectorField};
pub use format::{Codec, DType, FieldHeader, SegmentHeader};
pub use manifest::Manifest;
pub use meta::{
    GraphStoreConfig, PointStoreConfig, RedbCollectionStore, RedbGraphStore, RedbPointStore,
    open_graph_store, open_point_store,
};
pub use segment::{SegmentReader, SegmentWriter, field_dir, open_tier};
pub use tier::{BinaryTier, F16Tier, Int8Tier, PqTier};
pub use wal::{WalReader, WalWriter};
