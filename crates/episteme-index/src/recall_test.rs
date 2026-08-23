use super::*;
use episteme_core::Ordinal;

fn hits(rows: &[u32]) -> Vec<Candidate> {
    rows.iter()
        .enumerate()
        .map(|(i, &r)| Candidate::new(Ordinal::from_row(r), 1.0 - i as f32 * 0.01))
        .collect()
}

#[test]
fn perfect_agreement_is_one() {
    assert_eq!(recall_at_k(&hits(&[1, 2, 3]), &hits(&[1, 2, 3]), 3), 1.0);
}

#[test]
fn no_agreement_is_zero() {
    assert_eq!(recall_at_k(&hits(&[7, 8, 9]), &hits(&[1, 2, 3]), 3), 0.0);
}

#[test]
fn partial_agreement_is_proportional() {
    assert!((recall_at_k(&hits(&[1, 2, 9]), &hits(&[1, 2, 3]), 3) - 2.0 / 3.0).abs() < 1e-9);
}

#[test]
fn order_within_the_result_does_not_matter() {
    // A caller asked for the k nearest, not for them in a specific arrangement.
    assert_eq!(recall_at_k(&hits(&[3, 1, 2]), &hits(&[1, 2, 3]), 3), 1.0);
}

#[test]
fn extra_results_beyond_k_are_ignored() {
    assert_eq!(
        recall_at_k(&hits(&[1, 2, 3, 4, 5]), &hits(&[1, 2, 3]), 3),
        1.0
    );
}

#[test]
fn normalises_by_truth_not_by_k() {
    // Only two rows exist. Finding both is perfect, not 2/10.
    assert_eq!(recall_at_k(&hits(&[1, 2]), &hits(&[1, 2]), 10), 1.0);
}

#[test]
fn empty_truth_is_perfect_rather_than_a_division_by_zero() {
    assert_eq!(recall_at_k(&[], &[], 10), 1.0);
}

#[test]
fn a_report_tracks_the_worst_query_not_only_the_mean() {
    // A mean of 0.8 hides one query that returned nothing, and that query class
    // is usually the one users notice.
    let r = RecallReport::of(&[1.0, 1.0, 1.0, 0.0], 10);
    assert_eq!(r.queries, 4);
    assert_eq!(r.mean, 0.75);
    assert_eq!(r.worst, 0.0);
}

#[test]
fn thresholds_compare_against_the_mean() {
    let r = RecallReport::of(&[0.98, 0.96, 0.94], 10);
    assert!(r.meets(0.95), "mean 0.96 clears 0.95");
    assert!(!r.meets(0.99));
}

#[test]
fn a_threshold_exactly_at_the_mean_is_not_a_reliable_gate() {
    // Summing three values that "average to 0.95" lands at 0.9499999... — so a
    // CI gate set to the exact figure it was tuned against will flake. Gates
    // want headroom, not equality.
    let r = RecallReport::of(&[0.96, 0.95, 0.94], 10);
    assert!(
        r.mean < 0.95,
        "float summation lands just under: {}",
        r.mean
    );
}

#[test]
fn an_empty_report_does_not_divide_by_zero() {
    let r = RecallReport::of(&[], 10);
    assert_eq!(r.mean, 1.0);
    assert_eq!(r.queries, 0);
}

#[test]
fn display_is_readable_in_a_ci_log() {
    let rendered = RecallReport::of(&[0.98, 0.92], 10).to_string();
    assert!(rendered.contains("recall@10"));
    assert!(rendered.contains("worst"));
}
