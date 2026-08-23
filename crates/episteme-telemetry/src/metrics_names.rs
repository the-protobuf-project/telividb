//! Metric names and their units.
//!
//! Named per Prometheus convention — `_seconds`, `_bytes`, `_total` — because
//! the unit belongs in the name where a dashboard author will see it.
//!
//! Every name here is registered with a description at startup via
//! [`describe_all`], so `/metrics` is self-documenting rather than a list of
//! bare counters somebody has to guess at.

// --- query path -------------------------------------------------------------

/// Histogram. End-to-end search latency, planner through rerank.
pub const SEARCH_DURATION: &str = "episteme_search_duration_seconds";

/// Histogram. Vectors actually scored. The number that explains a slow query:
/// latency tracks this far more closely than it tracks `k`.
pub const SEARCH_CANDIDATES: &str = "episteme_search_candidates_visited";

/// Histogram. Results returned. Persistently below the requested `k` means a
/// filter is too selective for the strategy the planner chose.
pub const SEARCH_RESULTS: &str = "episteme_search_results_returned";

/// Counter. Result sets that were not complete — labelled by reason.
pub const SEARCH_INCOMPLETE: &str = "episteme_search_incomplete_total";

/// Histogram. Sampled recall against exhaustive search.
///
/// Recall is measured against brute force in CI, but that says nothing about
/// production, where the data distribution is different and drifts. Sampling a
/// small fraction of live queries and re-running them exactly, asynchronously,
/// is the only way to know. Almost nobody does this.
pub const SEARCH_RECALL: &str = "episteme_search_recall_at_k";

// --- write path -------------------------------------------------------------

/// Histogram. Time to flush and fsync one group commit — the write path's real cost.
pub const WAL_COMMIT_DURATION: &str = "episteme_wal_commit_duration_seconds";
/// Histogram. Records per group commit. Falling toward one means batching has stopped working.
pub const WAL_COMMIT_RECORDS: &str = "episteme_wal_commit_records";
/// Counter. Bytes appended to the log.
pub const WAL_BYTES: &str = "episteme_wal_bytes_total";

/// Counter. Torn tails found during recovery. Expected after a hard kill;
/// a rising rate in steady state means something is wrong with the host.
pub const WAL_TORN_RECOVERIES: &str = "episteme_wal_torn_recoveries_total";

/// Histogram. Time to seal a buffer into an immutable segment.
pub const SEGMENT_SEAL_DURATION: &str = "episteme_segment_seal_duration_seconds";
/// Histogram. Time to build an index over a sealed segment.
pub const INDEX_BUILD_DURATION: &str = "episteme_index_build_duration_seconds";
/// Histogram. Time to publish a new manifest generation.
pub const MANIFEST_SWAP_DURATION: &str = "episteme_manifest_swap_duration_seconds";
/// Histogram. Time to merge segments and drop tombstoned rows.
pub const COMPACTION_DURATION: &str = "episteme_compaction_duration_seconds";

// --- state ------------------------------------------------------------------

/// Gauge. Segments named by the current manifest.
pub const SEGMENTS_LIVE: &str = "episteme_segments_live";
/// Gauge. Rows visible to readers.
pub const ROWS_LIVE: &str = "episteme_rows_live";

/// Gauge. Tombstoned rows still occupying space. Rising without compaction
/// keeping up means reads are scanning bytes they will only discard.
pub const ROWS_TOMBSTONED: &str = "episteme_rows_tombstoned";

// --- embedding --------------------------------------------------------------

/// Histogram. Time to embed one batch.
pub const EMBED_DURATION: &str = "episteme_embed_duration_seconds";
/// Histogram. Inputs per embedding batch.
pub const EMBED_BATCH_SIZE: &str = "episteme_embed_batch_size";

// --- policy -----------------------------------------------------------------

/// Counter. Requests denied, by action. A spike is either an attack or a
/// misconfigured grant, and both want to be visible.
pub const POLICY_DENIED: &str = "episteme_policy_denied_total";

/// Histogram. Policy resolution time. Must stay flat as corpora grow — if it
/// tracks collection size, policy is being evaluated per row instead of once
/// per query, which is a design failure and not a slow path.
pub const POLICY_RESOLVE_DURATION: &str = "episteme_policy_resolve_duration_seconds";

// --- bulk jobs --------------------------------------------------------------

/// Counter. Bulk job records, labelled by outcome.
pub const JOB_RECORDS: &str = "episteme_job_records_total";
/// Histogram. Bulk job wall-clock duration.
pub const JOB_DURATION: &str = "episteme_job_duration_seconds";

/// Every metric with its description, for registration at startup.
pub const ALL: &[(&str, &str)] = &[
    (SEARCH_DURATION, "End-to-end search latency in seconds"),
    (SEARCH_CANDIDATES, "Vectors scored while answering a search"),
    (SEARCH_RESULTS, "Results returned by a search"),
    (
        SEARCH_INCOMPLETE,
        "Searches that returned an incomplete result set",
    ),
    (SEARCH_RECALL, "Sampled recall@k against exhaustive search"),
    (
        WAL_COMMIT_DURATION,
        "Time to flush and fsync a WAL group commit",
    ),
    (
        WAL_COMMIT_RECORDS,
        "Records included in one WAL group commit",
    ),
    (WAL_BYTES, "Bytes appended to the write-ahead log"),
    (WAL_TORN_RECOVERIES, "WAL recoveries that found a torn tail"),
    (
        SEGMENT_SEAL_DURATION,
        "Time to seal a segment and write its files",
    ),
    (
        INDEX_BUILD_DURATION,
        "Time to build an index over a sealed segment",
    ),
    (
        MANIFEST_SWAP_DURATION,
        "Time to publish a new manifest generation",
    ),
    (
        COMPACTION_DURATION,
        "Time to merge segments and drop tombstoned rows",
    ),
    (SEGMENTS_LIVE, "Segments currently in the manifest"),
    (ROWS_LIVE, "Rows visible to readers"),
    (
        ROWS_TOMBSTONED,
        "Rows tombstoned but not yet compacted away",
    ),
    (EMBED_DURATION, "Time to embed one batch"),
    (EMBED_BATCH_SIZE, "Inputs per embedding batch"),
    (POLICY_DENIED, "Requests denied by policy, by action"),
    (
        POLICY_RESOLVE_DURATION,
        "Time to resolve a principal to a visibility context",
    ),
    (JOB_RECORDS, "Bulk job records, by outcome"),
    (JOB_DURATION, "Bulk job wall-clock duration"),
];

#[cfg(test)]
#[path = "metrics_names_test.rs"]
mod tests;
