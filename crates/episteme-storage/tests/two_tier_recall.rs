//! Two-tier search with real codecs, measured against exhaustive search.
//!
//! The codec table says how much ranking each codec loses on its own. This asks
//! the question that actually matters: **how much of that loss does reranking
//! recover, and at what fraction of the corpus read at full precision.**
//!
//! Clustered data, matching real embeddings.

mod support;

use episteme_core::{ScanTier, VectorStore};
use episteme_index::{
    FlatIndex, OverFetch, VectorIndex, adapters::MemoryStore, recall_at_k, two_tier_search,
};
use episteme_storage::format::quantize::{PqCodebook, PqParams};
use episteme_storage::{BinaryTier, Int8Tier, PqTier};
use support::{DIM, ROWS, corpus};

/// Mean recall@10 of a two-tier search against exhaustive search, and the mean
/// fraction of the corpus that reached full precision.
fn measure(store: &MemoryStore, tier: &dyn ScanTier, queries: &[Vec<f32>]) -> (f64, f64) {
    let mut recalls = Vec::new();
    let mut fractions = Vec::new();

    for q in queries {
        let truth = FlatIndex.search(store, q, 10, None).unwrap();
        let (hits, stats) =
            two_tier_search(tier, store, q, 10, OverFetch::default(), None).unwrap();
        recalls.push(recall_at_k(&hits, &truth, 10));
        fractions.push(stats.rerank_fraction());
    }

    (
        recalls.iter().sum::<f64>() / recalls.len() as f64,
        fractions.iter().sum::<f64>() / fractions.len() as f64,
    )
}

#[test]
fn int8_two_tier_recovers_full_recall() {
    let (store, queries) = corpus();
    let tier = Int8Tier::build(&store);
    let (recall, fraction) = measure(&store, &tier, &queries);
    println!(
        "int8   recall {recall:.4}, reranked {:.1}% of corpus",
        fraction * 100.0
    );
    assert!(recall >= 0.99, "int8 two-tier recall {recall}");
}

#[test]
fn pq_needs_over_fetch_proportional_to_its_compression() {
    // The finding this test exists to record: **a coarser codec needs a wider
    // candidate set, not merely a rerank.**
    //
    // Reranking can only reorder what the coarse pass admitted. At m=8 the
    // reconstruction error on unit vectors is ~0.083, which is comparable to
    // the within-cluster spread of this corpus — so with the default 4x
    // over-fetch the true top-10 often never enter the candidate set at all,
    // and no amount of rescoring recovers them.
    //
    // Widening the over-fetch fixes it, and that is the correct lever: the
    // compression ratio and the over-fetch multiplier are a pair, and tuning
    // one without the other is how two-tier gets a bad reputation.
    let (store, queries) = corpus();
    let refs: Vec<&[f32]> = (0..store.len())
        .filter_map(|r| store.get(episteme_core::Ordinal::from_row(r as u32)))
        .collect();
    let book = PqCodebook::train(
        &refs,
        DIM,
        PqParams {
            m: 8,
            ..Default::default()
        },
    )
    .unwrap();
    let tier = PqTier::build(&store, book).unwrap();

    let (narrow, _) = measure(&store, &tier, &queries);

    let wide = OverFetch {
        multiplier: 20.0,
        minimum: 200,
    };
    let mut recalls = Vec::new();
    let mut fractions = Vec::new();
    for q in &queries {
        let truth = FlatIndex.search(&store, q, 10, None).unwrap();
        let (hits, stats) = two_tier_search(&tier, &store, q, 10, wide, None).unwrap();
        recalls.push(recall_at_k(&hits, &truth, 10));
        fractions.push(stats.rerank_fraction());
    }
    let recall = recalls.iter().sum::<f64>() / recalls.len() as f64;
    let fraction = fractions.iter().sum::<f64>() / fractions.len() as f64;

    println!(
        "pq     recall {narrow:.4} at 4x over-fetch, {recall:.4} at 20x \
         (reranked {:.1}% of corpus)",
        fraction * 100.0
    );
    assert!(
        recall > narrow,
        "wider over-fetch should recover recall: {narrow} -> {recall}"
    );
    assert!(recall >= 0.85, "pq two-tier recall at 20x: {recall}");
}

#[test]
fn binary_two_tier_turns_a_useless_ranking_into_a_usable_one() {
    // Binary alone agreed with exhaustive ranking about 27% of the time. It is
    // a pruning pass, and this is what it looks like used correctly.
    let (store, queries) = corpus();
    let tier = BinaryTier::build(&store);

    let wide = OverFetch {
        multiplier: 20.0,
        minimum: 200,
    };
    let mut recalls = Vec::new();
    for q in &queries {
        let truth = FlatIndex.search(&store, q, 10, None).unwrap();
        let (hits, _) = two_tier_search(&tier, &store, q, 10, wide, None).unwrap();
        recalls.push(recall_at_k(&hits, &truth, 10));
    }
    let recall = recalls.iter().sum::<f64>() / recalls.len() as f64;
    println!("binary recall {recall:.4} at 20x over-fetch");
    assert!(recall > 0.50, "binary two-tier recall {recall}");
}

#[test]
fn most_of_the_corpus_never_reaches_full_precision() {
    // The saving, stated as a property rather than left implicit.
    let (store, queries) = corpus();
    let tier = Int8Tier::build(&store);
    let (_, fraction) = measure(&store, &tier, &queries);
    assert!(fraction < 0.05, "reranked {fraction} of the corpus");
}

#[test]
fn tiers_report_their_size_honestly() {
    let (store, _) = corpus();
    let raw = ROWS * DIM * 4;

    let int8 = Int8Tier::build(&store).bytes();
    let binary = BinaryTier::build(&store).bytes();
    assert!(int8 < raw / 3, "int8 {int8} vs raw {raw}");
    assert!(binary < raw / 30, "binary {binary} vs raw {raw}");
}

#[test]
fn a_filtered_two_tier_search_matches_a_filtered_exhaustive_one() {
    let (store, queries) = corpus();
    let tier = Int8Tier::build(&store);
    let keep_even = |o: episteme_core::Ordinal| o.row().is_multiple_of(2);

    for q in queries.iter().take(10) {
        let truth = FlatIndex.search(&store, q, 10, Some(&keep_even)).unwrap();
        let (hits, _) =
            two_tier_search(&tier, &store, q, 10, OverFetch::default(), Some(&keep_even)).unwrap();
        assert!(hits.iter().all(|h| h.ordinal.row().is_multiple_of(2)));
        assert!(recall_at_k(&hits, &truth, 10) >= 0.9);
    }
}
