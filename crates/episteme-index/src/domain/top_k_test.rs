use super::*;
use episteme_core::Ordinal;

fn candidate(row: u32, score: f32) -> Candidate {
    Candidate::new(Ordinal::from_row(row), score)
}

fn collect(k: usize, higher_is_nearer: bool, scores: &[f32]) -> Vec<(u32, f32)> {
    let mut top = TopK::new(k, higher_is_nearer);
    for (row, &score) in scores.iter().enumerate() {
        top.offer(candidate(row as u32, score));
    }
    top.into_sorted()
        .into_iter()
        .map(|c| (c.ordinal.row(), c.score))
        .collect()
}

#[test]
fn keeps_the_highest_when_higher_is_nearer() {
    let got = collect(3, true, &[0.1, 0.9, 0.5, 0.7, 0.2]);
    assert_eq!(
        got.iter().map(|(_, s)| *s).collect::<Vec<_>>(),
        vec![0.9, 0.7, 0.5]
    );
}

#[test]
fn keeps_the_lowest_when_lower_is_nearer() {
    // L2 distance: smaller is better, and getting this backwards returns the
    // furthest rows while looking entirely healthy.
    let got = collect(3, false, &[0.1, 0.9, 0.5, 0.7, 0.2]);
    assert_eq!(
        got.iter().map(|(_, s)| *s).collect::<Vec<_>>(),
        vec![0.1, 0.2, 0.5]
    );
}

#[test]
fn holds_only_k_however_many_are_offered() {
    // The whole point: the heap must not grow with the corpus.
    let scores: Vec<f32> = (0..10_000).map(|i| i as f32).collect();
    assert_eq!(collect(5, true, &scores).len(), 5);
}

#[test]
fn fewer_candidates_than_k_are_all_kept() {
    assert_eq!(collect(10, true, &[0.3, 0.1, 0.2]).len(), 3);
}

#[test]
fn zero_k_keeps_nothing() {
    assert!(collect(0, true, &[0.3, 0.1]).is_empty());
}

#[test]
fn matches_a_full_sort_exactly() {
    // The property that makes this a safe replacement for collect-then-sort:
    // same results, same order, for both metric directions.
    let scores: Vec<f32> = (0..500)
        .map(|i| ((i * 7919) % 1000) as f32 / 1000.0)
        .collect();

    for higher_is_nearer in [true, false] {
        let mut sorted: Vec<Candidate> = scores
            .iter()
            .enumerate()
            .map(|(row, &s)| candidate(row as u32, s))
            .collect();
        if higher_is_nearer {
            sorted.sort_unstable_by(|a, b| {
                b.score
                    .total_cmp(&a.score)
                    .then(a.ordinal.row().cmp(&b.ordinal.row()))
            });
        } else {
            sorted.sort_unstable_by(|a, b| {
                a.score
                    .total_cmp(&b.score)
                    .then(a.ordinal.row().cmp(&b.ordinal.row()))
            });
        }
        let expected: Vec<f32> = sorted.iter().take(20).map(|c| c.score).collect();
        let got: Vec<f32> = collect(20, higher_is_nearer, &scores)
            .into_iter()
            .map(|(_, s)| s)
            .collect();
        assert_eq!(got, expected, "higher_is_nearer = {higher_is_nearer}");
    }
}

#[test]
fn ties_are_broken_deterministically() {
    // Two rows with the same score must not depend on heap internals for which
    // survives, or the same query returns different rows between runs.
    let scores = vec![0.5f32; 100];
    let first = collect(5, true, &scores);
    let second = collect(5, true, &scores);
    assert_eq!(first, second);
}
