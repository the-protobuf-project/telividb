use super::*;
use crate::adapters::MemoryStore;
use episteme_core::{Dim, Metric};

fn store_of(metric: Metric, rows: &[&[f32]]) -> MemoryStore {
    let mut s = MemoryStore::new(Dim::new(rows[0].len() as u32).unwrap(), metric);
    for r in rows {
        s.push(r).unwrap();
    }
    s
}

fn cand(row: u32, score: f32) -> Candidate {
    Candidate::new(Ordinal::from_row(row), score)
}

#[test]
fn over_fetch_scales_with_k() {
    let f = OverFetch::default();
    assert_eq!(f.candidates_for(100), 400);
}

#[test]
fn over_fetch_respects_a_floor_for_small_k() {
    // At k=1 a 4x multiplier gives four candidates, which is far too thin a
    // set for reranking to recover anything.
    let f = OverFetch::default();
    assert_eq!(f.candidates_for(1), 32);
}

#[test]
fn over_fetch_never_returns_fewer_than_k() {
    let f = OverFetch {
        multiplier: 0.5,
        minimum: 1,
    };
    assert_eq!(
        f.candidates_for(50),
        50,
        "cannot fetch fewer than requested"
    );
}

#[test]
fn reranking_corrects_a_coarse_ordering() {
    // The whole point: the coarse tier ranked row 2 first, full precision says
    // row 0. That correction is what two-tier buys.
    let store = store_of(Metric::Dot, &[&[1.0, 0.0], &[0.5, 0.0], &[0.1, 0.0]]);
    let coarse = vec![cand(2, 9.0), cand(1, 8.0), cand(0, 7.0)];

    let out = rerank(&store, &[1.0, 0.0], &coarse, 3);
    let order: Vec<u32> = out.iter().map(|c| c.ordinal.row()).collect();
    assert_eq!(order, vec![0, 1, 2]);
}

#[test]
fn reranking_honours_ascending_metrics() {
    let store = store_of(Metric::L2, &[&[9.0, 9.0], &[1.0, 1.0]]);
    let out = rerank(&store, &[0.0, 0.0], &[cand(0, 0.0), cand(1, 0.0)], 2);
    assert_eq!(out[0].ordinal.row(), 1, "nearest by L2 comes first");
}

#[test]
fn scores_returned_are_full_precision_not_the_coarse_ones() {
    let store = store_of(Metric::Dot, &[&[2.0, 0.0]]);
    let out = rerank(&store, &[3.0, 0.0], &[cand(0, 0.123)], 1);
    assert_eq!(out[0].score, 6.0, "must report the rescored value");
}

#[test]
fn truncates_to_k_after_rescoring_not_before() {
    // Truncating first would discard the row that full precision ranks best.
    let store = store_of(Metric::Dot, &[&[0.1, 0.0], &[0.2, 0.0], &[9.0, 0.0]]);
    let coarse = vec![cand(0, 5.0), cand(1, 4.0), cand(2, 0.1)];
    let out = rerank(&store, &[1.0, 0.0], &coarse, 1);
    assert_eq!(out[0].ordinal.row(), 2);
}

#[test]
fn absent_rows_are_dropped_not_carried_at_their_coarse_score() {
    let mut store = MemoryStore::new(Dim::new(2).unwrap(), Metric::Dot);
    store.push(&[1.0, 0.0]).unwrap();
    store.push_absent();

    let out = rerank(&store, &[1.0, 0.0], &[cand(1, 99.0), cand(0, 0.1)], 5);
    assert_eq!(out.len(), 1, "the absent row cannot be ranked");
    assert_eq!(out[0].ordinal.row(), 0);
}

#[test]
fn an_empty_candidate_set_yields_nothing() {
    let store = store_of(Metric::Dot, &[&[1.0, 0.0]]);
    assert!(rerank(&store, &[1.0, 0.0], &[], 10).is_empty());
}

#[test]
fn stats_report_how_much_the_order_changed() {
    let store = store_of(Metric::Dot, &[&[1.0, 0.0], &[0.5, 0.0]]);
    let coarse = vec![cand(1, 9.0), cand(0, 8.0)];
    let (out, stats) = rerank_measured(&store, &[1.0, 0.0], &coarse, 2);

    assert_eq!(out[0].ordinal.row(), 0);
    assert_eq!(stats.considered, 2);
    assert_eq!(stats.returned, 2);
    assert_eq!(stats.reordered, 2, "both positions changed occupant");
}

#[test]
fn stats_report_zero_when_the_coarse_order_was_already_right() {
    // Near-zero reordering means the over-fetch is doing no work and could be
    // reduced; near-total means the coarse tier is too lossy to prune with.
    let store = store_of(Metric::Dot, &[&[1.0, 0.0], &[0.5, 0.0]]);
    let coarse = vec![cand(0, 9.0), cand(1, 8.0)];
    let (_, stats) = rerank_measured(&store, &[1.0, 0.0], &coarse, 2);
    assert_eq!(stats.reordered, 0);
}
