use super::*;

fn cand(row: u32, score: f32) -> Candidate {
    Candidate::new(Ordinal::from_row(row), score)
}

#[test]
fn empty_input_yields_nothing() {
    let m = merge_top_k(&[], 10, true);
    assert!(m.hits.is_empty());
    assert_eq!(m.stats.sources_searched, 0);
}

#[test]
fn k_zero_yields_nothing_but_still_counts_sources() {
    let m = merge_top_k(&[(Source::Buffer, vec![cand(0, 0.9)])], 0, true);
    assert!(m.hits.is_empty());
    assert_eq!(m.stats.sources_searched, 1);
}

#[test]
fn selects_across_sources_not_within_them() {
    // The property that matters: taking k from each source and concatenating
    // would return 0.90 and 0.10 for k=2. Merging correctly returns 0.90, 0.80.
    let m = merge_top_k(
        &[
            (Source::Buffer, vec![cand(0, 0.90), cand(1, 0.80)]),
            (Source::Sealed(1), vec![cand(0, 0.10), cand(1, 0.05)]),
        ],
        2,
        true,
    );
    let scores: Vec<f32> = m.hits.iter().map(|h| h.score).collect();
    assert_eq!(scores, vec![0.90, 0.80]);
}

#[test]
fn interleaves_sources_by_score() {
    let m = merge_top_k(
        &[
            (Source::Buffer, vec![cand(0, 0.9), cand(1, 0.5)]),
            (Source::Sealed(7), vec![cand(0, 0.7), cand(1, 0.3)]),
        ],
        4,
        true,
    );
    let order: Vec<(Source, f32)> = m.hits.iter().map(|h| (h.source, h.score)).collect();
    assert_eq!(
        order,
        vec![
            (Source::Buffer, 0.9),
            (Source::Sealed(7), 0.7),
            (Source::Buffer, 0.5),
            (Source::Sealed(7), 0.3),
        ]
    );
}

#[test]
fn honours_ascending_metrics() {
    // L2: lower is nearer. Ranking descending here would return the worst k.
    let m = merge_top_k(
        &[
            (Source::Buffer, vec![cand(0, 9.0)]),
            (Source::Sealed(1), vec![cand(0, 1.0), cand(1, 5.0)]),
        ],
        2,
        false,
    );
    let scores: Vec<f32> = m.hits.iter().map(|h| h.score).collect();
    assert_eq!(scores, vec![1.0, 5.0]);
}

#[test]
fn attributes_hits_to_their_source() {
    let m = merge_top_k(
        &[
            (Source::Buffer, vec![cand(0, 0.9)]),
            (Source::Sealed(1), vec![cand(0, 0.8), cand(1, 0.7)]),
        ],
        3,
        true,
    );
    assert_eq!(m.stats.buffer_hits, 1);
    assert_eq!(m.stats.sealed_hits, 2);
    assert_eq!(m.stats.sources_searched, 2);
}

#[test]
fn attribution_counts_returned_hits_not_candidates() {
    // Truncation must be reflected: three buffer candidates, one survives k=1.
    let m = merge_top_k(
        &[(
            Source::Buffer,
            vec![cand(0, 0.9), cand(1, 0.8), cand(2, 0.7)],
        )],
        1,
        true,
    );
    assert_eq!(m.hits.len(), 1);
    assert_eq!(m.stats.buffer_hits, 1, "must count returned, not scanned");
}

#[test]
fn buffer_only_results_are_fully_exact() {
    // Such an answer says nothing about index quality. Counting it as a recall
    // sample would drag measured recall toward 1.0 and mask a degraded index.
    let m = merge_top_k(&[(Source::Buffer, vec![cand(0, 0.9)])], 5, true);
    assert!(m.stats.is_fully_exact());
}

#[test]
fn any_sealed_hit_makes_the_answer_approximate() {
    let m = merge_top_k(
        &[
            (Source::Buffer, vec![cand(0, 0.9)]),
            (Source::Sealed(3), vec![cand(0, 0.1)]),
        ],
        5,
        true,
    );
    assert!(!m.stats.is_fully_exact());
}

#[test]
fn source_exactness_is_declared() {
    assert!(Source::Buffer.is_exact(), "the buffer scan is exhaustive");
    assert!(!Source::Sealed(1).is_exact());
}

#[test]
fn k_larger_than_available_returns_everything() {
    let m = merge_top_k(
        &[
            (Source::Buffer, vec![cand(0, 0.9)]),
            (Source::Sealed(1), vec![cand(0, 0.5)]),
        ],
        99,
        true,
    );
    assert_eq!(m.hits.len(), 2);
}

#[test]
fn an_empty_source_still_counts_as_searched() {
    let m = merge_top_k(
        &[
            (Source::Buffer, vec![]),
            (Source::Sealed(1), vec![cand(0, 0.5)]),
        ],
        5,
        true,
    );
    assert_eq!(m.stats.sources_searched, 2);
    assert_eq!(m.stats.buffer_hits, 0);
    assert_eq!(m.hits.len(), 1);
}

#[test]
fn identical_scores_from_different_sources_both_survive() {
    let m = merge_top_k(
        &[
            (Source::Buffer, vec![cand(0, 0.5)]),
            (Source::Sealed(1), vec![cand(0, 0.5)]),
        ],
        2,
        true,
    );
    assert_eq!(m.hits.len(), 2, "a tie must not silently drop one");
    assert_eq!(m.stats.buffer_hits + m.stats.sealed_hits, 2);
}
