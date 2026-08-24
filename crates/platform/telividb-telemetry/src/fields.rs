//! The span-field vocabulary.
//!
//! One name per concept, used everywhere. Add here rather than inventing a key
//! at a call site — a field spelled two ways is two fields, and nothing joins
//! them back together.

/// Collection the operation targets. **Label-safe** — bounded by how many
/// collections exist.
pub const COLLECTION: &str = "telividb.collection";

/// Named vector field, e.g. `text_bge`. **Label-safe** — bounded by schema.
pub const FIELD: &str = "telividb.field";

/// Index algorithm: `flat`, `hnsw`, `ivfpq`, `gpu-flat`. **Label-safe.**
pub const INDEX_KIND: &str = "telividb.index.kind";

/// Where a GPU-resident index actually landed: `metal`, `cuda`, `cpu`.
/// **Label-safe** — a closed set of three.
///
/// Worth emitting on every build and search: an index that silently fell back
/// to CPU is otherwise indistinguishable from a working GPU one, and would
/// pass every correctness test while delivering none of the speed.
pub const DEVICE: &str = "telividb.index.device";

/// Distance metric: `dot`, `l2`, `cosine`. **Label-safe.**
pub const METRIC: &str = "telividb.metric";

/// Storage codec for the scan tier. **Label-safe.**
pub const CODEC: &str = "telividb.codec";

/// Filter strategy the planner chose: `prefilter`, `traversal`, `postfilter`.
/// **Label-safe** — and the field you need to explain a slow query.
pub const STRATEGY: &str = "telividb.query.strategy";

/// Which store backend answered: `redb`, `postgres`, `arango`.
/// **Label-safe** — a closed set, one per adapter that exists.
pub const BACKEND: &str = "telividb.store.backend";

/// Which store was opened: `point`, `graph`. **Label-safe** — a closed set.
///
/// Deliberately *not* a path. A store's path is built from a collection's
/// resource name, and a collection may be a vault — emitting one would
/// disclose that vault's existence, which is what rule 28 forbids. The kind of
/// store is safe; which collection it belongs to travels separately, through
/// [`RESOURCE`] after [`redact::resource_token`](crate::redact::resource_token).
pub const STORE: &str = "telividb.store.kind";

/// What kind of thing is holding memory: `point-store`, `graph-store`,
/// `vector-index`, `model`. **Label-safe** — a closed set.
pub const RESIDENT_KIND: &str = "telividb.resident.kind";

/// Where resident memory sits: `host` or `device`. **Label-safe.**
///
/// The dimension that matters for a model zoo: it is what separates an index
/// competing for VRAM from a store that merely occupies disk and page cache.
pub const LOCATION: &str = "telividb.resident.location";

/// Bytes a resident thing holds. A measurement, so span-only.
pub const RESIDENT_BYTES: &str = "telividb.resident.bytes";

/// Graph edge type, e.g. `HAS_SHOT`. **Label-safe** — bounded by schema, the
/// same way [`FIELD`] is.
pub const EDGE_TYPE: &str = "telividb.graph.edge_type";

/// Hop limit on a traversal. Span-only: it is caller-supplied and unbounded.
pub const HOPS: &str = "telividb.graph.hops";

/// Number of graph nodes, whether rehydrated or reached by a traversal.
pub const NODES: &str = "telividb.graph.nodes";

/// Why a result set was incomplete: `shard_timeout`, `vault_locked`.
/// **Label-safe.**
pub const INCOMPLETE_REASON: &str = "telividb.incomplete.reason";

// ---------------------------------------------------------------------------
// Span-only. Unbounded or high-cardinality — never a metric label.
// ---------------------------------------------------------------------------

/// Segment identifier. Grows without bound as segments are created.
pub const SEGMENT_ID: &str = "telividb.segment.id";

/// Manifest generation. Monotonic, so unbounded.
pub const GENERATION: &str = "telividb.manifest.generation";

/// Bulk job identifier.
pub const JOB_ID: &str = "telividb.job.id";

/// Principal performing the operation. **Emit only as an opaque id**, and treat
/// it as sensitive: a record of who searched for what is surveillance data.
pub const PRINCIPAL: &str = "telividb.principal";

/// Resource name of the target. Must pass through [`crate::redact`] first.
pub const RESOURCE: &str = "telividb.resource";

// ---------------------------------------------------------------------------
// Measurements
// ---------------------------------------------------------------------------

/// Requested number of results.
pub const K: &str = "telividb.query.k";
/// Vectors actually scored while answering. Latency tracks this far more closely than it tracks `k`.
pub const CANDIDATES_VISITED: &str = "telividb.query.candidates_visited";
/// Results returned. Persistently below `k` means a filter is too selective for the chosen strategy.
pub const RESULTS_RETURNED: &str = "telividb.query.results_returned";
/// Row count of whatever the span covers.
pub const ROWS: &str = "telividb.rows";
/// Byte count of whatever the span covers.
pub const BYTES: &str = "telividb.bytes";
/// Records a recovery could not restore — an undecodable payload, or one the
/// buffer refused. Emitted beside [`RECORDS`] so a lossy replay is visible
/// rather than looking complete.
pub const REJECTED: &str = "telividb.rejected";
/// Record count, for WAL commits and bulk jobs.
pub const RECORDS: &str = "telividb.records";
/// Vector width. Safe to emit — shape discloses nothing.
pub const DIM: &str = "telividb.dim";
/// Wall-clock duration of whatever the record covers, in seconds.
///
/// Carried on the log record as well as in a histogram: the histogram is
/// what a dashboard reads, and this is what someone reading one operation's
/// log line needs in order to see why it was slow.
pub const DURATION_SECONDS: &str = "telividb.duration_seconds";
/// Byte offset within a file — where a torn WAL tail was found.
pub const OFFSET: &str = "telividb.offset";
/// Segment count of whatever the record covers.
pub const SEGMENTS: &str = "telividb.segments";
/// Rows written by a compaction.
pub const ROWS_WRITTEN: &str = "telividb.rows_written";
/// Rows dropped by a compaction because they were tombstoned.
pub const ROWS_RECLAIMED: &str = "telividb.rows_reclaimed";
/// Whether a search carried an authorization or attribute filter.
pub const FILTERED: &str = "telividb.query.filtered";
/// The query, **as a shape only** — never values. See [`crate::redact`].
pub const QUERY: &str = "telividb.query.shape";
/// Edge count of a built graph.
pub const EDGES: &str = "telividb.index.edges";
/// Layer count of a built graph.
pub const LEVELS: &str = "telividb.index.levels";
/// Search breadth an HNSW query actually used.
pub const EF: &str = "telividb.query.ef";

/// Field names that are safe to use as **metric labels** because their value
/// space is bounded by schema or by a closed enum.
///
/// Anything absent from this list is span-only. This is checked in tests rather
/// than left to reviewer memory.
pub const LABEL_SAFE: &[&str] = &[
    COLLECTION,
    FIELD,
    INDEX_KIND,
    DEVICE,
    METRIC,
    CODEC,
    STRATEGY,
    BACKEND,
    STORE,
    RESIDENT_KIND,
    LOCATION,
    EDGE_TYPE,
    INCOMPLETE_REASON,
];

/// Field names that must never be emitted at all. Present so the rule is
/// testable, not merely documented.
pub const FORBIDDEN: &[&str] = &[
    "telividb.query.vector",
    "telividb.payload",
    "telividb.vault.name",
    "telividb.content.text",
];

#[cfg(test)]
#[path = "fields_test.rs"]
mod tests;
