use super::*;

fn relevant(pairs: &[(&str, u32)]) -> HashMap<String, u32> {
    pairs.iter().map(|(id, g)| ((*id).to_owned(), *g)).collect()
}

fn ranked(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn a_perfect_ranking_scores_one() {
    let rel = relevant(&[("a", 1), ("b", 1)]);
    assert!((ndcg_at_k(&ranked(&["a", "b", "c"]), &rel, 10) - 1.0).abs() < 1e-9);
}

#[test]
fn ordering_changes_ndcg_even_when_recall_is_identical() {
    // The property nDCG exists for. Both retrieve the same document, so recall
    // is equal; only the rank differs.
    let rel = relevant(&[("a", 1)]);
    let early = ndcg_at_k(&ranked(&["a", "x", "y"]), &rel, 10);
    let late = ndcg_at_k(&ranked(&["x", "y", "a"]), &rel, 10);

    assert!(early > late, "early {early} should beat late {late}");
    assert_eq!(
        recall_at_k(&ranked(&["a", "x", "y"]), &rel, 10),
        recall_at_k(&ranked(&["x", "y", "a"]), &rel, 10),
    );
}

#[test]
fn a_higher_grade_outranks_a_lower_one() {
    // The exponential gain has to actually use the grade, or graded qrels
    // would score the same as boolean ones.
    let rel = relevant(&[("a", 2), ("b", 1)]);
    let best = ndcg_at_k(&ranked(&["a", "b"]), &rel, 10);
    let swapped = ndcg_at_k(&ranked(&["b", "a"]), &rel, 10);
    assert!(best > swapped, "grade order ignored: {best} vs {swapped}");
}

#[test]
fn retrieving_nothing_relevant_scores_zero() {
    let rel = relevant(&[("a", 1)]);
    assert_eq!(ndcg_at_k(&ranked(&["x", "y"]), &rel, 10), 0.0);
    assert_eq!(recall_at_k(&ranked(&["x", "y"]), &rel, 10), 0.0);
}

#[test]
fn a_query_with_no_judgements_contributes_zero_rather_than_dividing_by_zero() {
    let rel = relevant(&[]);
    assert_eq!(ndcg_at_k(&ranked(&["a"]), &rel, 10), 0.0);
    assert_eq!(recall_at_k(&ranked(&["a"]), &rel, 10), 0.0);
}

#[test]
fn cutoff_is_respected() {
    // A document below the cutoff must not count, or every k would report the
    // same number.
    let rel = relevant(&[("a", 1)]);
    assert_eq!(recall_at_k(&ranked(&["x", "y", "a"]), &rel, 2), 0.0);
    assert_eq!(recall_at_k(&ranked(&["x", "y", "a"]), &rel, 3), 1.0);
}

#[test]
fn a_report_averages_across_queries() {
    let mut report = Report::default();
    report.add(&ranked(&["a"]), &relevant(&[("a", 1)]));
    report.add(&ranked(&["x"]), &relevant(&[("a", 1)]));
    let report = report.finish();

    assert_eq!(report.queries, 2);
    assert!(
        (report.ndcg_at_10 - 0.5).abs() < 1e-9,
        "got {}",
        report.ndcg_at_10
    );
}
