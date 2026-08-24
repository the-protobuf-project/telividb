use super::PqTier;
use crate::format::quantize::{PqCodebook, PqParams};
use episteme_core::{Metric, Ordinal, PreparedQuery, ScanTier};

/// A codebook over `dim` floats in `m` subspaces, trained on enough rows.
fn codebook(dim: usize, m: usize) -> PqCodebook {
    let rows: Vec<Vec<f32>> = (0..300)
        .map(|i| (0..dim).map(|d| ((i * 31 + d * 7) % 97) as f32).collect())
        .collect();
    let refs: Vec<&[f32]> = rows.iter().map(Vec::as_slice).collect();
    PqCodebook::train(
        &refs,
        dim,
        PqParams {
            m,
            ..Default::default()
        },
    )
    .expect("trained")
}

/// A tier over `rows` encoded rows.
fn tier(dim: usize, m: usize, rows: usize) -> PqTier {
    let book = codebook(dim, m);
    let mut codes = Vec::new();
    for i in 0..rows {
        let v: Vec<f32> = (0..dim).map(|d| ((i * 13 + d) % 89) as f32).collect();
        codes.extend_from_slice(&book.encode(&v).expect("encode"));
    }
    PqTier::from_codes(&codes, book, rows, &|_| true).expect("tier")
}

#[test]
fn a_short_distance_table_returns_none_rather_than_panicking() {
    // `PreparedQuery::table` is public and takes an arbitrary vector, so a
    // table shorter than `subspaces * CENTROIDS` reaches `score`. Indexing it
    // raw aborted the process; a caller-supplied length must not be able to do
    // that.
    let tier = tier(16, 4, 8);
    let truncated = PreparedQuery::table(Metric::Dot, 4, vec![0.0; 4]);
    assert_eq!(tier.score(&truncated, Ordinal::from_row(0)), None);
}

#[test]
fn an_empty_distance_table_returns_none() {
    let tier = tier(16, 4, 8);
    let empty = PreparedQuery::table(Metric::Dot, 4, Vec::new());
    assert_eq!(tier.score(&empty, Ordinal::from_row(0)), None);
}

#[test]
fn a_properly_prepared_query_still_scores() {
    // The bounds check must not have turned every score into `None` — that
    // would be the same silent-empty-result failure, arrived at differently.
    let tier = tier(16, 4, 8);
    let query: Vec<f32> = (0..16).map(|d| d as f32).collect();
    let prepared = tier.prepare(&query, Metric::Dot).expect("prepare");
    assert!(
        tier.score(&prepared, Ordinal::from_row(0)).is_some(),
        "a correctly prepared query scored nothing"
    );
}

#[test]
fn a_row_past_the_end_is_absent_rather_than_a_panic() {
    let tier = tier(16, 4, 8);
    let query: Vec<f32> = (0..16).map(|d| d as f32).collect();
    let prepared = tier.prepare(&query, Metric::Dot).expect("prepare");
    assert_eq!(tier.score(&prepared, Ordinal::from_row(99)), None);
}
