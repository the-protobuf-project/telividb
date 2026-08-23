use super::*;
use episteme_core::Fingerprint;

fn header(rows: u64, deleted: u64) -> SegmentHeader {
    SegmentHeader {
        schema_fingerprint: Fingerprint::of(b"schema"),
        rows,
        deleted,
    }
}

#[test]
fn a_clean_collection_needs_no_compaction() {
    let segments = vec![(1, header(50_000, 0)), (2, header(50_000, 10))];
    assert_eq!(plan(&segments, CompactionPolicy::default()), None);
}

#[test]
fn heavy_tombstones_trigger_a_rewrite() {
    let segments = vec![(1, header(1_000, 400))];
    let p = plan(&segments, CompactionPolicy::default()).unwrap();
    assert_eq!(p.inputs, vec![1]);
    assert_eq!(p.surviving_rows, 600);
    assert_eq!(p.reclaimed_rows, 400);
    assert!(p.is_worthwhile());
}

#[test]
fn tombstone_pressure_is_checked_before_segment_count() {
    // A segment mostly full of deleted rows costs every query; many small
    // segments cost only fan-out. The worse problem goes first.
    let segments = vec![
        (1, header(1_000, 500)),
        (2, header(10, 0)),
        (3, header(10, 0)),
        (4, header(10, 0)),
        (5, header(10, 0)),
    ];
    let p = plan(&segments, CompactionPolicy::default()).unwrap();
    assert_eq!(p.reason, "tombstone ratio above threshold");
    assert_eq!(p.inputs, vec![1]);
}

#[test]
fn many_small_segments_are_merged() {
    let segments: Vec<_> = (1..=4).map(|i| (i, header(100, 0))).collect();
    let p = plan(&segments, CompactionPolicy::default()).unwrap();
    assert_eq!(p.reason, "too many small segments");
    assert_eq!(p.inputs.len(), 4);
    assert_eq!(p.surviving_rows, 400);
}

#[test]
fn too_few_small_segments_are_left_alone() {
    let segments: Vec<_> = (1..=3).map(|i| (i, header(100, 0))).collect();
    assert_eq!(plan(&segments, CompactionPolicy::default()), None);
}

#[test]
fn an_empty_segment_does_not_divide_by_zero() {
    let segments = vec![(1, header(0, 0))];
    let _ = plan(&segments, CompactionPolicy::default());
}

#[test]
fn a_stricter_policy_triggers_earlier() {
    let segments = vec![(1, header(1_000, 50))];
    assert_eq!(plan(&segments, CompactionPolicy::default()), None);

    let strict = CompactionPolicy {
        tombstone_ratio: 0.01,
        ..Default::default()
    };
    assert!(plan(&segments, strict).is_some());
}

#[test]
fn an_empty_collection_needs_nothing() {
    assert_eq!(plan(&[], CompactionPolicy::default()), None);
}

#[test]
fn a_tombstone_heavy_plan_is_capped() {
    // Every tombstone-heavy segment used to go into one plan. With hundreds of
    // them that is a single unbounded rewrite: it reads the whole database,
    // publishes nothing until the end, and cannot be resumed partway.
    let policy = CompactionPolicy {
        max_inputs: 8,
        ..CompactionPolicy::default()
    };
    let segments: Vec<(u64, SegmentHeader)> = (0..200)
        .map(|id| {
            let mut header = SegmentHeader::new(Fingerprint::unset(), 1_000);
            header.deleted = 900;
            (id, header)
        })
        .collect();

    let plan = plan(&segments, policy).expect("tombstone pressure is a reason to compact");
    assert_eq!(plan.inputs.len(), 8, "the plan was not capped");
}

#[test]
fn the_worst_segments_are_compacted_first() {
    // A capped plan must spend its budget where it repays most, or the next
    // run keeps picking up the same marginal segments.
    let policy = CompactionPolicy {
        max_inputs: 2,
        ..CompactionPolicy::default()
    };
    let segments: Vec<(u64, SegmentHeader)> = [(0u64, 300u64), (1, 900), (2, 500), (3, 250)]
        .into_iter()
        .map(|(id, deleted)| {
            let mut header = SegmentHeader::new(Fingerprint::unset(), 1_000);
            header.deleted = deleted;
            (id, header)
        })
        .collect();

    let plan = plan(&segments, policy).expect("a plan");
    assert_eq!(plan.inputs, vec![1, 2], "expected the two most tombstoned");
}

#[test]
fn a_capped_plan_is_reproducible() {
    // Two segments with the same ratio must break the tie the same way every
    // run, or a retried compaction rewrites a different set than it planned.
    let policy = CompactionPolicy {
        max_inputs: 2,
        ..CompactionPolicy::default()
    };
    let segments: Vec<(u64, SegmentHeader)> = (0..6)
        .map(|id| {
            let mut header = SegmentHeader::new(Fingerprint::unset(), 1_000);
            header.deleted = 500;
            (id, header)
        })
        .collect();

    let first = plan(&segments, policy).expect("a plan");
    let second = plan(&segments, policy).expect("a plan");
    assert_eq!(first.inputs, second.inputs);
}
