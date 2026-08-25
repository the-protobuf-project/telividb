use super::*;
use crate::adapters::MemoryStore;
use telividb_core::{Dim, Metric, PreparedQuery, PreparedState};
use telividb_distance::Scorer;

/// A scan tier that scores exactly, so tests can isolate the composition from
/// any particular codec's error.
struct ExactTier {
    rows: Vec<Option<Vec<f32>>>,
}

impl ExactTier {
    fn of(rows: &[Option<&[f32]>]) -> Self {
        Self {
            rows: rows.iter().map(|r| r.map(|v| v.to_vec())).collect(),
        }
    }
}

impl ScanTier for ExactTier {
    fn codec(&self) -> &'static str {
        "exact-test"
    }
    fn len(&self) -> usize {
        self.rows.len()
    }
    fn prepare(&self, query: &[f32], metric: Metric) -> Result<PreparedQuery> {
        Ok(PreparedQuery::vector(metric, query.to_vec()))
    }
    fn score(&self, prepared: &PreparedQuery, ordinal: Ordinal) -> Option<f32> {
        let row = self.rows.get(ordinal.row() as usize)?.as_ref()?;
        let PreparedState::Vector(q) = &prepared.state else {
            return None;
        };
        Some(prepared.metric.score(q, row))
    }
}

/// A tier whose ordering is deliberately wrong, to prove the rerank corrects it.
struct ReversedTier(ExactTier);

impl ScanTier for ReversedTier {
    fn codec(&self) -> &'static str {
        "reversed-test"
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn prepare(&self, query: &[f32], metric: Metric) -> Result<PreparedQuery> {
        self.0.prepare(query, metric)
    }
    fn score(&self, prepared: &PreparedQuery, ordinal: Ordinal) -> Option<f32> {
        self.0.score(prepared, ordinal).map(|s| -s)
    }
}

fn store_of(rows: &[&[f32]]) -> MemoryStore {
    let mut s = MemoryStore::new(Dim::new(rows[0].len() as u32).unwrap(), Metric::Dot);
    for r in rows {
        s.push(r).unwrap();
    }
    s
}

#[test]
fn returns_the_true_nearest_when_both_tiers_agree() {
    let rows: Vec<&[f32]> = vec![&[0.1, 0.0], &[1.0, 0.0], &[0.5, 0.0]];
    let exact = store_of(&rows);
    let tier = ExactTier::of(&rows.iter().map(|r| Some(*r)).collect::<Vec<_>>());

    let (hits, stats) = search(&tier, &exact, &[1.0, 0.0], 2, OverFetch::default(), None).unwrap();
    assert_eq!(hits[0].ordinal.row(), 1);
    assert_eq!(hits[1].ordinal.row(), 2);
    assert_eq!(stats.scanned, 3);
    assert_eq!(stats.returned, 2);
}

#[test]
fn the_rerank_corrects_a_wrong_coarse_ordering() {
    // The whole point of two-tier: a lossy scan may order badly, and full
    // precision fixes it as long as the true answer entered the candidate set.
    let rows: Vec<&[f32]> = vec![&[0.1, 0.0], &[1.0, 0.0], &[0.5, 0.0]];
    let exact = store_of(&rows);
    let tier = ReversedTier(ExactTier::of(
        &rows.iter().map(|r| Some(*r)).collect::<Vec<_>>(),
    ));

    let (hits, stats) = search(&tier, &exact, &[1.0, 0.0], 3, OverFetch::default(), None).unwrap();
    assert_eq!(hits[0].ordinal.row(), 1, "rerank must fix the order");
    assert!(stats.reordered > 0, "the correction should be reported");
}

#[test]
fn scores_returned_are_exact_not_coarse() {
    let rows: Vec<&[f32]> = vec![&[2.0, 0.0]];
    let exact = store_of(&rows);
    let tier = ReversedTier(ExactTier::of(&[Some(&[2.0, 0.0])]));

    let (hits, _) = search(&tier, &exact, &[3.0, 0.0], 1, OverFetch::default(), None).unwrap();
    assert_eq!(hits[0].score, 6.0, "must report the full-precision score");
}

#[test]
fn the_filter_is_applied_during_the_scan() {
    // Filtering afterwards would leak how many rows were hidden and where they
    // ranked. Excluding row 1 must promote row 2, not leave a gap.
    let rows: Vec<&[f32]> = vec![&[0.1, 0.0], &[1.0, 0.0], &[0.9, 0.0]];
    let exact = store_of(&rows);
    let tier = ExactTier::of(&rows.iter().map(|r| Some(*r)).collect::<Vec<_>>());
    let deny_one = |o: Ordinal| o.row() != 1;

    let (hits, stats) = search(
        &tier,
        &exact,
        &[1.0, 0.0],
        2,
        OverFetch::default(),
        Some(&deny_one),
    )
    .unwrap();

    assert_eq!(stats.scanned, 2, "the excluded row must not be scored");
    assert_eq!(hits.len(), 2, "k is still filled from visible rows");
    assert!(hits.iter().all(|h| h.ordinal.row() != 1));
}

#[test]
fn absent_rows_are_skipped_by_both_tiers() {
    let mut exact = MemoryStore::new(Dim::new(2).unwrap(), Metric::Dot);
    exact.push(&[1.0, 0.0]).unwrap();
    exact.push_absent();

    let tier = ExactTier::of(&[Some(&[1.0, 0.0]), None]);
    let (hits, stats) = search(&tier, &exact, &[1.0, 0.0], 5, OverFetch::default(), None).unwrap();

    assert_eq!(stats.scanned, 1);
    assert_eq!(hits.len(), 1);
}

#[test]
fn k_zero_does_no_work() {
    let exact = store_of(&[&[1.0, 0.0]]);
    let tier = ExactTier::of(&[Some(&[1.0, 0.0])]);
    let (hits, stats) = search(&tier, &exact, &[1.0, 0.0], 0, OverFetch::default(), None).unwrap();
    assert!(hits.is_empty());
    assert_eq!(stats.scanned, 0, "must not scan for k=0");
}

#[test]
fn over_fetch_bounds_what_reaches_full_precision() {
    // The saving two-tier exists for: most of the corpus is rejected without
    // ever being read at full precision.
    let rows: Vec<Vec<f32>> = (0..100).map(|i| vec![i as f32 / 100.0, 0.0]).collect();
    let refs: Vec<&[f32]> = rows.iter().map(Vec::as_slice).collect();
    let exact = store_of(&refs);
    let tier = ExactTier::of(&refs.iter().map(|r| Some(*r)).collect::<Vec<_>>());

    let (_, stats) = search(
        &tier,
        &exact,
        &[1.0, 0.0],
        5,
        OverFetch {
            multiplier: 4.0,
            minimum: 20,
        },
        None,
    )
    .unwrap();

    assert_eq!(stats.scanned, 100);
    assert_eq!(stats.candidates, 20);
    assert!(
        stats.rerank_fraction() < 0.25,
        "most rows avoided full precision"
    );
}

#[test]
fn a_dimension_mismatch_is_rejected_before_scanning() {
    let exact = store_of(&[&[1.0, 0.0]]);
    let tier = ExactTier::of(&[Some(&[1.0, 0.0])]);
    assert!(search(&tier, &exact, &[1.0], 1, OverFetch::default(), None).is_err());
}

#[test]
fn rerank_fraction_is_zero_when_nothing_was_scanned() {
    assert_eq!(TwoTierStats::default().rerank_fraction(), 0.0);
}
