//! The span-field vocabulary.
//!
//! One name per concept, used everywhere. Add here rather than inventing a key
//! at a call site — a field spelled two ways is two fields, and nothing joins
//! them back together.

/// Collection the operation targets. **Label-safe** — bounded by how many
/// collections exist.
pub const COLLECTION: &str = "episteme.collection";

/// Named vector field, e.g. `text_bge`. **Label-safe** — bounded by schema.
pub const FIELD: &str = "episteme.field";

/// Index algorithm: `flat`, `hnsw`, `ivfpq`. **Label-safe.**
pub const INDEX_KIND: &str = "episteme.index.kind";

/// Distance metric: `dot`, `l2`, `cosine`. **Label-safe.**
pub const METRIC: &str = "episteme.metric";

/// Storage codec for the scan tier. **Label-safe.**
pub const CODEC: &str = "episteme.codec";

/// Filter strategy the planner chose: `prefilter`, `traversal`, `postfilter`.
/// **Label-safe** — and the field you need to explain a slow query.
pub const STRATEGY: &str = "episteme.query.strategy";

/// Why a result set was incomplete: `shard_timeout`, `vault_locked`.
/// **Label-safe.**
pub const INCOMPLETE_REASON: &str = "episteme.incomplete.reason";

// ---------------------------------------------------------------------------
// Span-only. Unbounded or high-cardinality — never a metric label.
// ---------------------------------------------------------------------------

/// Segment identifier. Grows without bound as segments are created.
pub const SEGMENT_ID: &str = "episteme.segment.id";

/// Manifest generation. Monotonic, so unbounded.
pub const GENERATION: &str = "episteme.manifest.generation";

/// Bulk job identifier.
pub const JOB_ID: &str = "episteme.job.id";

/// Principal performing the operation. **Emit only as an opaque id**, and treat
/// it as sensitive: a record of who searched for what is surveillance data.
pub const PRINCIPAL: &str = "episteme.principal";

/// Resource name of the target. Must pass through [`crate::redact`] first.
pub const RESOURCE: &str = "episteme.resource";

// ---------------------------------------------------------------------------
// Measurements
// ---------------------------------------------------------------------------

/// Requested number of results.
pub const K: &str = "episteme.query.k";
/// Vectors actually scored while answering. Latency tracks this far more closely than it tracks `k`.
pub const CANDIDATES_VISITED: &str = "episteme.query.candidates_visited";
/// Results returned. Persistently below `k` means a filter is too selective for the chosen strategy.
pub const RESULTS_RETURNED: &str = "episteme.query.results_returned";
/// Row count of whatever the span covers.
pub const ROWS: &str = "episteme.rows";
/// Byte count of whatever the span covers.
pub const BYTES: &str = "episteme.bytes";
/// Record count, for WAL commits and bulk jobs.
pub const RECORDS: &str = "episteme.records";
/// Vector width. Safe to emit — shape discloses nothing.
pub const DIM: &str = "episteme.dim";
/// Wall-clock duration of whatever the record covers, in seconds.
///
/// Carried on the log record as well as in a histogram: the histogram is
/// what a dashboard reads, and this is what someone reading one operation's
/// log line needs in order to see why it was slow.
pub const DURATION_SECONDS: &str = "episteme.duration_seconds";
/// Byte offset within a file — where a torn WAL tail was found.
pub const OFFSET: &str = "episteme.offset";
/// Segment count of whatever the record covers.
pub const SEGMENTS: &str = "episteme.segments";
/// Rows written by a compaction.
pub const ROWS_WRITTEN: &str = "episteme.rows_written";
/// Rows dropped by a compaction because they were tombstoned.
pub const ROWS_RECLAIMED: &str = "episteme.rows_reclaimed";
/// Whether a search carried an authorization or attribute filter.
pub const FILTERED: &str = "episteme.query.filtered";
/// The query, **as a shape only** — never values. See [`crate::redact`].
pub const QUERY: &str = "episteme.query.shape";
/// Edge count of a built graph.
pub const EDGES: &str = "episteme.index.edges";
/// Layer count of a built graph.
pub const LEVELS: &str = "episteme.index.levels";
/// Search breadth an HNSW query actually used.
pub const EF: &str = "episteme.query.ef";

/// Field names that are safe to use as **metric labels** because their value
/// space is bounded by schema or by a closed enum.
///
/// Anything absent from this list is span-only. This is checked in tests rather
/// than left to reviewer memory.
pub const LABEL_SAFE: &[&str] = &[
    COLLECTION,
    FIELD,
    INDEX_KIND,
    METRIC,
    CODEC,
    STRATEGY,
    INCOMPLETE_REASON,
];

/// Field names that must never be emitted at all. Present so the rule is
/// testable, not merely documented.
pub const FORBIDDEN: &[&str] = &[
    "episteme.query.vector",
    "episteme.payload",
    "episteme.vault.name",
    "episteme.content.text",
];

#[cfg(test)]
#[path = "fields_test.rs"]
mod tests;
