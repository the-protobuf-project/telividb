//! The ggml encoder against the candle one, on the same model and the same text.
//!
//! **This is the gate on removing candle.** A rewritten encoder that runs, loads
//! and returns finite vectors of the right width can still be wrong in every
//! way that matters — a wrong RoPE convention, a transposed QKV split, a mask
//! applied after the softmax — and none of those fail a smoke test. The only
//! check that means anything is agreement with the implementation whose recall
//! was measured against a published benchmark.
//!
//! Skipped when the model is absent (80 MiB, not committed). A skip says so
//! out loud rather than passing quietly.

use std::path::PathBuf;
use telividb_embed::domain::Pooling;

/// Cosine similarity, the metric these vectors are actually used under.
///
/// Compared rather than element-wise equality because the two runtimes reduce
/// in different orders and quantize differently — bit-identical output is not
/// the claim. Direction is.
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (na * nb).max(f32::MIN_POSITIVE)
}

fn model() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/models/gguf/text/nomic-embed-text-v1.5.Q4_K_M.gguf");
    path.exists().then_some(path)
}

/// The reference vector, recorded from the implementation whose recall was
/// measured against BEIR.
///
/// A committed fixture rather than a live comparison, because the reference
/// implementation is being removed — and a correctness guarantee that
/// disappears with the thing it was checking is not a guarantee. Regenerate
/// Re-establishing it is a deliberate act needing a reference implementation:
/// the fixture records which one produced it, and the candle encoder is in git
/// history. Nothing in this repository regenerates it automatically, because a
/// test that rewrites its own expectation proves nothing.
fn reference() -> (Vec<u32>, Vec<u32>, Vec<f32>) {
    let raw = include_str!("fixtures/nomic_cls.json");
    let v: serde_json::Value = serde_json::from_str(raw).expect("fixture parses");
    let take = |key: &str| -> Vec<u32> {
        v[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as u32)
            .collect()
    };
    let vector = v["vector"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_f64().unwrap() as f32)
        .collect();
    (take("ids"), take("attention"), vector)
}

#[test]
fn the_encoder_reproduces_the_recorded_reference_vector() {
    let Some(path) = model() else {
        eprintln!("SKIPPED: run examples/models/download.sh to exercise this");
        return;
    };

    use telividb_compute::Backend;
    use telividb_embed::adapters::ggml::Encoder;

    // The *same* backend candle picks, so the comparison isolates the
    // implementation rather than conflating it with cross-backend numerics.
    let backend = Backend::best().expect("a backend");
    let encoder = Encoder::load(&path, backend).expect("ggml encoder loads");

    let (ids, attention, expected) = reference();

    let out = encoder
        .forward(&ids, &attention, 1, Pooling::Mean)
        .expect("forward runs");

    assert_eq!(out.len(), 1, "one vector per row");
    assert_eq!(
        out[0].len(),
        encoder.config().hidden,
        "vector width must be the model's hidden size"
    );
    assert!(
        out[0].iter().all(|v| v.is_finite()),
        "a non-finite value means a NaN reached the pool — usually an all-masked softmax"
    );
    // A vector of all zeros is what an unwired graph produces, and it would
    // pass every check above.
    let magnitude: f32 = out[0].iter().map(|v| v.abs()).sum();
    assert!(magnitude > 0.0, "the encoder returned an all-zero vector");

    // The measurement that actually decides whether this encoder is correct.
    let cls = encoder
        .forward(&ids, &attention, 1, Pooling::Cls)
        .expect("cls forward");
    let agreement = cosine(&cls[0], &expected);
    eprintln!("ggml vs recorded reference: cosine {agreement:.6}");
    assert!(
        agreement > 0.99,
        "the encoder no longer reproduces the reference vector: cosine {agreement}"
    );
}

#[test]
fn padding_does_not_change_a_row_s_vector() {
    let Some(path) = model() else {
        eprintln!("SKIPPED: run examples/models/download.sh to exercise this");
        return;
    };

    use telividb_compute::Backend;
    use telividb_embed::adapters::ggml::Encoder;

    let backend = Backend::best().expect("a backend");
    let encoder = Encoder::load(&path, backend).expect("ggml encoder loads");

    // The same three tokens, once alone and once padded to six. If the mask is
    // applied after the softmax instead of before, or if the mean divides by
    // the padded width, these diverge — and the divergence depends on what the
    // row was batched with, which is why it is so hard to notice in production.
    let tight = encoder
        .forward(&[101, 2023, 102], &[1, 1, 1], 1, Pooling::Mean)
        .expect("tight batch");
    let padded = encoder
        .forward(
            &[101, 2023, 102, 0, 0, 0],
            &[1, 1, 1, 0, 0, 0],
            1,
            Pooling::Mean,
        )
        .expect("padded batch");

    let agreement = cosine(&tight[0], &padded[0]);
    assert!(
        agreement > 0.999,
        "padding changed the vector: cosine {agreement}"
    );
}
