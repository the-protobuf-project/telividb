use super::*;

#[test]
fn every_field_is_namespaced() {
    let all = [
        COLLECTION,
        FIELD,
        INDEX_KIND,
        METRIC,
        CODEC,
        STRATEGY,
        INCOMPLETE_REASON,
        SEGMENT_ID,
        GENERATION,
        JOB_ID,
        PRINCIPAL,
        RESOURCE,
        K,
        CANDIDATES_VISITED,
        RESULTS_RETURNED,
        ROWS,
        BYTES,
        RECORDS,
        DIM,
    ];
    for f in all {
        assert!(f.starts_with("episteme."), "{f} is not namespaced");
    }
}

#[test]
fn unbounded_fields_are_not_label_safe() {
    // The cardinality rule, enforced rather than remembered. Each of these
    // grows without bound, and as a metric label would multiply time series
    // until the monitoring system falls over.
    for f in [SEGMENT_ID, GENERATION, JOB_ID, PRINCIPAL, RESOURCE] {
        assert!(!LABEL_SAFE.contains(&f), "{f} must not be a metric label");
    }
}

#[test]
fn measurements_are_not_label_safe() {
    // Values, not dimensions. As labels they would be catastrophic.
    for f in [
        K,
        CANDIDATES_VISITED,
        RESULTS_RETURNED,
        ROWS,
        BYTES,
        RECORDS,
        DIM,
    ] {
        assert!(
            !LABEL_SAFE.contains(&f),
            "{f} is a measurement, not a label"
        );
    }
}

#[test]
fn forbidden_fields_are_never_label_safe() {
    for f in FORBIDDEN {
        assert!(!LABEL_SAFE.contains(f), "{f} must never be emitted at all");
    }
}

#[test]
fn label_safe_set_is_deduplicated() {
    let mut sorted = LABEL_SAFE.to_vec();
    sorted.sort_unstable();
    let before = sorted.len();
    sorted.dedup();
    assert_eq!(before, sorted.len(), "duplicate entry in LABEL_SAFE");
}
