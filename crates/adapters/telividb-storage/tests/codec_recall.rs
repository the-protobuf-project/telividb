//! What each codec costs in ranking quality.
//!
//! Compression ratios are easy to quote and meaningless alone — the number that
//! matters is how often a codec reorders neighbours relative to full precision.
//! These tests pin that down so a change to a codec cannot quietly trade recall
//! for bytes.
//!
//! Data is clustered, matching real embeddings. Uniform noise has no structure
//! for a codebook to capture, so it would understate PQ badly.

use telividb_storage::format::quantize::{BinaryCodes, F16Row, Int8Row, PqCodebook, PqParams};

const DIM: usize = 64;
const ROWS: usize = 400;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 40) as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }
}

fn corpus() -> Vec<Vec<f32>> {
    let mut rng = Rng(42);
    let centres: Vec<Vec<f32>> = (0..16)
        .map(|_| (0..DIM).map(|_| rng.next()).collect())
        .collect();
    (0..ROWS)
        .map(|i| {
            centres[i % centres.len()]
                .iter()
                .map(|c| c + rng.next() * 0.1)
                .collect()
        })
        .collect()
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Fraction of the true top-10 that a codec's ranking also returns.
fn top_k_agreement(rows: &[Vec<f32>], decoded: &[Vec<f32>], queries: &[Vec<f32>]) -> f64 {
    let mut total = 0.0;
    for q in queries {
        let rank = |vs: &[Vec<f32>]| {
            let mut scored: Vec<(usize, f32)> =
                vs.iter().enumerate().map(|(i, v)| (i, dot(q, v))).collect();
            scored.sort_by(|a, b| b.1.total_cmp(&a.1));
            scored
                .into_iter()
                .take(10)
                .map(|(i, _)| i)
                .collect::<Vec<_>>()
        };
        let truth = rank(rows);
        let approx = rank(decoded);
        let hits = approx.iter().filter(|i| truth.contains(i)).count();
        total += hits as f64 / truth.len() as f64;
    }
    total / queries.len() as f64
}

fn queries() -> Vec<Vec<f32>> {
    let mut rng = Rng(7);
    (0..20)
        .map(|_| (0..DIM).map(|_| rng.next()).collect())
        .collect()
}

#[test]
fn f16_is_effectively_lossless_for_ranking() {
    let rows = corpus();
    let decoded: Vec<Vec<f32>> = rows.iter().map(|v| F16Row::encode(v).decode()).collect();
    let agreement = top_k_agreement(&rows, &decoded, &queries());
    println!("f16 agreement {agreement:.4}");
    assert!(agreement > 0.99, "f16 lost ranking quality: {agreement}");
}

#[test]
fn int8_keeps_almost_all_ranking_at_four_fold_compression() {
    let rows = corpus();
    let decoded: Vec<Vec<f32>> = rows.iter().map(|v| Int8Row::encode(v).decode()).collect();
    let agreement = top_k_agreement(&rows, &decoded, &queries());
    println!("int8 agreement {agreement:.4}");
    assert!(agreement > 0.95, "int8 lost too much ranking: {agreement}");
}

#[test]
fn pq_keeps_useful_ranking_at_eight_fold_compression() {
    let rows = corpus();
    let refs: Vec<&[f32]> = rows.iter().map(Vec::as_slice).collect();
    let book = PqCodebook::train(
        &refs,
        DIM,
        PqParams {
            m: 8,
            ..Default::default()
        },
    )
    .unwrap();

    let decoded: Vec<Vec<f32>> = rows
        .iter()
        .map(|v| book.decode(&book.encode(v).unwrap()).unwrap())
        .collect();

    let agreement = top_k_agreement(&rows, &decoded, &queries());
    println!("pq(m=8) agreement {agreement:.4}");
    assert!(agreement > 0.60, "pq lost too much ranking: {agreement}");
}

#[test]
fn binary_is_coarse_enough_to_need_a_rerank() {
    // Documents the trade rather than asserting quality: binary is a pruning
    // pass, and using it alone returns plausible neighbours that are wrong.
    let rows = corpus();
    let decoded: Vec<Vec<f32>> = rows
        .iter()
        .map(|v| BinaryCodes::encode(v).decode())
        .collect();
    let agreement = top_k_agreement(&rows, &decoded, &queries());
    println!("binary agreement {agreement:.4}");
    assert!(agreement > 0.05, "binary should still prune usefully");
}

#[test]
fn accuracy_ranks_as_the_compression_ratios_predict() {
    // The ordering the storage design depends on. If this ever inverts, a codec
    // is broken rather than merely lossy.
    let rows = corpus();
    let q = queries();
    let f16 = top_k_agreement(
        &rows,
        &rows
            .iter()
            .map(|v| F16Row::encode(v).decode())
            .collect::<Vec<_>>(),
        &q,
    );
    let int8 = top_k_agreement(
        &rows,
        &rows
            .iter()
            .map(|v| Int8Row::encode(v).decode())
            .collect::<Vec<_>>(),
        &q,
    );
    let binary = top_k_agreement(
        &rows,
        &rows
            .iter()
            .map(|v| BinaryCodes::encode(v).decode())
            .collect::<Vec<_>>(),
        &q,
    );

    assert!(f16 >= int8, "f16 {f16} should be at least int8 {int8}");
    assert!(int8 > binary, "int8 {int8} should beat binary {binary}");
}
