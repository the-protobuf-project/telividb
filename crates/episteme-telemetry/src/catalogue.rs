//! The metric catalogue: every name, its instrument kind and its description.
//!
//! Separate from [`crate::metrics_names`] so the constants stay readable as a
//! list. The names are what call sites reference; this is what documents them.

use crate::metrics_names::*;
use telemetry::metrics::MetricType;

/// Every metric with its instrument kind and description.
///
/// The kind decides which instrument the stack records through. Getting it
/// wrong is not cosmetic: a counter stored as a histogram cannot be rated,
/// and a gauge stored as a counter reads as a total that only ever climbs.
pub const ALL: &[(&str, MetricType, &str)] = &[
    (
        SEARCH_DURATION,
        MetricType::Histogram,
        "End-to-end search latency in seconds",
    ),
    (
        SEARCH_CANDIDATES,
        MetricType::Histogram,
        "Vectors scored while answering a search",
    ),
    (
        SEARCH_RESULTS,
        MetricType::Histogram,
        "Results returned by a search",
    ),
    (
        SEARCH_INCOMPLETE,
        MetricType::Counter,
        "Searches that returned an incomplete result set",
    ),
    (
        SEARCH_RECALL,
        MetricType::Histogram,
        "Sampled recall@k against exhaustive search",
    ),
    (
        WAL_COMMIT_DURATION,
        MetricType::Histogram,
        "Time to flush and fsync a WAL group commit",
    ),
    (
        WAL_COMMIT_RECORDS,
        MetricType::Histogram,
        "Records included in one WAL group commit",
    ),
    (
        WAL_BYTES,
        MetricType::Counter,
        "Bytes appended to the write-ahead log",
    ),
    (
        WAL_TORN_RECOVERIES,
        MetricType::Counter,
        "WAL recoveries that found a torn tail",
    ),
    (
        SEGMENT_SEAL_DURATION,
        MetricType::Histogram,
        "Time to seal a segment and write its files",
    ),
    (
        INDEX_BUILD_DURATION,
        MetricType::Histogram,
        "Time to build an index over a sealed segment",
    ),
    (
        MANIFEST_SWAP_DURATION,
        MetricType::Histogram,
        "Time to publish a new manifest generation",
    ),
    (
        COMPACTION_DURATION,
        MetricType::Histogram,
        "Time to merge segments and drop tombstoned rows",
    ),
    (
        SEGMENTS_LIVE,
        MetricType::Gauge,
        "Segments currently in the manifest",
    ),
    (ROWS_LIVE, MetricType::Gauge, "Rows visible to readers"),
    (
        ROWS_TOMBSTONED,
        MetricType::Gauge,
        "Rows tombstoned but not yet compacted away",
    ),
    (
        EMBED_DURATION,
        MetricType::Histogram,
        "Time to embed one batch",
    ),
    (
        EMBED_BATCH_SIZE,
        MetricType::Histogram,
        "Inputs per embedding batch",
    ),
    (
        POLICY_DENIED,
        MetricType::Counter,
        "Requests denied by policy, by action",
    ),
    (
        POLICY_RESOLVE_DURATION,
        MetricType::Histogram,
        "Time to resolve a principal to a visibility context",
    ),
    (
        JOB_RECORDS,
        MetricType::Counter,
        "Bulk job records, by outcome",
    ),
    (
        JOB_DURATION,
        MetricType::Histogram,
        "Bulk job wall-clock duration",
    ),
];

#[cfg(test)]
#[path = "catalogue_test.rs"]
mod tests;
