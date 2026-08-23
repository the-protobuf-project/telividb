use super::*;

#[test]
fn every_metric_is_described() {
    let all = [
        SEARCH_DURATION,
        SEARCH_CANDIDATES,
        SEARCH_RESULTS,
        SEARCH_INCOMPLETE,
        SEARCH_RECALL,
        WAL_COMMIT_DURATION,
        WAL_COMMIT_RECORDS,
        WAL_BYTES,
        WAL_TORN_RECOVERIES,
        SEGMENT_SEAL_DURATION,
        INDEX_BUILD_DURATION,
        MANIFEST_SWAP_DURATION,
        COMPACTION_DURATION,
        SEGMENTS_LIVE,
        ROWS_LIVE,
        ROWS_TOMBSTONED,
        EMBED_DURATION,
        EMBED_BATCH_SIZE,
        POLICY_DENIED,
        POLICY_RESOLVE_DURATION,
        JOB_RECORDS,
        JOB_DURATION,
    ];
    for name in all {
        assert!(
            ALL.iter().any(|(n, _)| *n == name),
            "{name} is missing from ALL, so /metrics will not document it"
        );
    }
}

#[test]
fn names_are_prefixed() {
    for (name, _) in ALL {
        assert!(name.starts_with("episteme_"), "{name} is not prefixed");
    }
}

#[test]
fn descriptions_are_present_and_useful() {
    for (name, desc) in ALL {
        assert!(!desc.is_empty(), "{name} has no description");
        assert!(desc.len() > 12, "{name}: description too thin to help");
    }
}

#[test]
fn units_are_declared_in_the_name() {
    for (name, _) in ALL {
        let ok = name.ends_with("_seconds")
            || name.ends_with("_bytes")
            || name.ends_with("_total")
            || name.ends_with("_live")
            || name.ends_with("_tombstoned")
            || name.ends_with("_records")
            || name.ends_with("_size")
            || name.ends_with("_visited")
            || name.ends_with("_returned")
            || name.ends_with("_at_k");
        assert!(ok, "{name} does not declare its unit in the name");
    }
}

#[test]
fn no_duplicate_names() {
    let mut names: Vec<_> = ALL.iter().map(|(n, _)| *n).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate metric name in ALL");
}
