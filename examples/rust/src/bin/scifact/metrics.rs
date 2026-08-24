//! The retrieval metrics BEIR reports.
//!
//! Implemented here rather than approximated, because the whole point of
//! running a standard dataset is that the number is comparable to a published
//! one. A metric that is "basically nDCG" is comparable to nothing.

use std::collections::HashMap;

/// Normalized discounted cumulative gain at `k`.
///
/// The primary BEIR metric. It rewards putting relevant documents *early*,
/// not merely retrieving them — which is what a search result's order is for,
/// and what plain recall cannot see.
///
/// `retrieved` is the ranked list of corpus ids, best first. `relevant` maps a
/// corpus id to its graded relevance.
pub fn ndcg_at_k(retrieved: &[String], relevant: &HashMap<String, u32>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }

    let dcg: f64 = retrieved
        .iter()
        .take(k)
        .enumerate()
        .map(|(rank, id)| gain(relevant.get(id).copied().unwrap_or(0), rank))
        .sum();

    // The best achievable ordering: every judgement, most relevant first.
    let mut ideal: Vec<u32> = relevant.values().copied().collect();
    ideal.sort_unstable_by(|a, b| b.cmp(a));
    let idcg: f64 = ideal
        .into_iter()
        .take(k)
        .enumerate()
        .map(|(rank, grade)| gain(grade, rank))
        .sum();

    match idcg > 0.0 {
        true => dcg / idcg,
        // No positive judgement at all — the query contributes nothing rather
        // than a division by zero.
        false => 0.0,
    }
}

/// One position's contribution: `(2^grade - 1) / log2(rank + 2)`.
///
/// The exponential form, which is what BEIR and the MTEB leaderboard use. The
/// linear alternative gives different numbers for the same ranking, so mixing
/// them would make a comparison meaningless.
fn gain(grade: u32, rank: usize) -> f64 {
    let numerator = (2f64.powi(grade as i32)) - 1.0;
    numerator / ((rank as f64) + 2.0).log2()
}

/// Fraction of the relevant documents that appear in the top `k`.
///
/// Reported beside nDCG because they fail differently: an encoder that finds
/// the right documents but orders them badly shows a healthy recall and a poor
/// nDCG, which points at pooling or normalisation rather than at the model.
pub fn recall_at_k(retrieved: &[String], relevant: &HashMap<String, u32>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let found = retrieved
        .iter()
        .take(k)
        .filter(|id| relevant.contains_key(*id))
        .count();
    found as f64 / relevant.len() as f64
}

/// Averaged scores across every query.
#[derive(Debug, Default, Clone, Copy)]
pub struct Report {
    /// Mean nDCG@10 — the headline, and what published figures quote.
    pub ndcg_at_10: f64,
    /// Mean Recall@10 — how much of the answer the first page holds.
    pub recall_at_10: f64,
    /// Mean Recall@100 — the ceiling a reranker could work up to,
    /// and what separates a ranking problem from an encoding one.
    pub recall_at_100: f64,
    /// How many queries were scored.
    pub queries: usize,
}

impl Report {
    /// Average one query's scores into the report.
    pub fn add(&mut self, retrieved: &[String], relevant: &HashMap<String, u32>) {
        self.ndcg_at_10 += ndcg_at_k(retrieved, relevant, 10);
        self.recall_at_10 += recall_at_k(retrieved, relevant, 10);
        self.recall_at_100 += recall_at_k(retrieved, relevant, 100);
        self.queries += 1;
    }

    /// Divide the running totals by the query count.
    pub fn finish(mut self) -> Self {
        if self.queries > 0 {
            let n = self.queries as f64;
            self.ndcg_at_10 /= n;
            self.recall_at_10 /= n;
            self.recall_at_100 /= n;
        }
        self
    }
}

#[cfg(test)]
#[path = "metrics_test.rs"]
mod tests;
