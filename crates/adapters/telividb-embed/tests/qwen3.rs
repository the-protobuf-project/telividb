//! The causal forward pass, against a real Qwen3-Embedding model.
//!
//! There is no committed reference vector for this family the way there is for
//! nomic — so this asks the question a reference would answer, differently:
//! **does the model behave like an embedder?**
//!
//! That is not a weak check. Every bug this pass could plausibly have — the
//! wrong norm, post-norm instead of pre-norm, keys tiled wrongly across query
//! heads, a head width derived as `hidden / heads` instead of read from the
//! header, CLS pooling where the model wants its last token, WordPiece over a
//! BPE vocabulary — produces finite vectors of the right width that no longer
//! separate related text from unrelated text. Semantic ordering is precisely
//! what they destroy.
//!
//! Skipped when the model is absent (639 MiB, not committed), and the skip
//! says so out loud rather than passing quietly.

use std::path::PathBuf;
use telividb_core::Fingerprint;
use telividb_embed::{GgmlInferencer, Inferencer, ModelId, Task};

/// Cosine similarity, the metric these vectors are used under.
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (na * nb).max(f32::MIN_POSITIVE)
}

fn model() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/models/gguf/text/Qwen3-Embedding-0.6B-Q8_0.gguf");
    path.exists().then_some(path)
}

#[test]
fn qwen3_loads_and_ranks_the_right_document_first() {
    let Some(path) = model() else {
        eprintln!("SKIPPED: Qwen3-Embedding-0.6B-Q8_0.gguf is not present");
        return;
    };

    let id = ModelId::new("qwen3-embedding-0.6b", Fingerprint::unset());
    let mut server = GgmlInferencer::new();
    server.register(&id, &path).expect("qwen3 should load");

    // The header says 1024 wide with 16 heads of 128 — deliberately wider heads
    // than the residual stream. Deriving head_dim as hidden/heads gives 64 and
    // misaligns every reshape, so this is the first thing to fail if that
    // regressed.
    assert_eq!(server.dim(&id).expect("a width").get(), 1024);

    // Ranking rather than an absolute margin. Decoder-derived embeddings are
    // anisotropic: everything sits at a high cosine to everything, so a
    // threshold on the raw number measures the family rather than the
    // implementation. What has to hold is the ordering — and every bug this
    // pass could have destroys it. Bidirectional attention on a causal model
    // did exactly that, scoring an unrelated sentence *above* a paraphrase.
    let documents = [
        "A kitten rested on the rug.".to_owned(),
        "Quarterly revenue exceeded analyst expectations.".to_owned(),
        "The compiler emits an error for unreachable patterns.".to_owned(),
        "Rainfall this spring was the heaviest on record.".to_owned(),
    ];
    let query = "The cat sat on the mat.".to_owned();

    let doc_vectors = server
        .embed(&id, Task::Document, &documents)
        .expect("documents should embed");
    let query_vector = server
        .embed(&id, Task::Query, std::slice::from_ref(&query))
        .expect("the query should embed");

    for (text, v) in documents.iter().zip(&doc_vectors) {
        assert_eq!(v.len(), 1024, "{text}");
        assert!(v.iter().all(|x| x.is_finite()), "{text}: non-finite output");
        assert!(
            v.iter().any(|x| x.abs() > 1e-6),
            "{text}: an all-zero vector means the graph ran and carried nothing"
        );
    }

    let mut scored: Vec<(f32, &String)> = doc_vectors
        .iter()
        .zip(&documents)
        .map(|(v, text)| (cosine(&query_vector[0], v), text))
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (score, text) in &scored {
        eprintln!("  {score:.4}  {text}");
    }

    assert_eq!(
        scored[0].1, &documents[0],
        "the paraphrase should rank first against a query about a cat; it ranked \
         {:?}. The model loaded and produced well-formed vectors that carry no \
         usable meaning, which is what every plausible bug in this forward pass \
         looks like.",
        scored[0].1
    );
}

#[test]
fn the_same_text_embeds_identically_twice() {
    // Batching pads to the longest row, so a vector that depends on what it was
    // batched with is a mask bug — and with last-token pooling it would mean
    // reading a padding position.
    let Some(path) = model() else {
        eprintln!("SKIPPED: Qwen3-Embedding-0.6B-Q8_0.gguf is not present");
        return;
    };

    let id = ModelId::new("qwen3-embedding-0.6b", Fingerprint::unset());
    let mut server = GgmlInferencer::new();
    server.register(&id, &path).expect("load");

    let alone = server
        .embed(&id, Task::Document, &["The cat sat on the mat.".to_owned()])
        .expect("alone");
    let batched = server
        .embed(
            &id,
            Task::Document,
            &[
                "The cat sat on the mat.".to_owned(),
                "A considerably longer sentence, included purely so that the \
                 batch has to pad the shorter one to reach it."
                    .to_owned(),
            ],
        )
        .expect("batched");

    let agreement = cosine(&alone[0], &batched[0]);
    assert!(
        agreement > 0.999,
        "the same text embedded {agreement:.6} against itself depending on its \
         batch, so padding is reaching the pooled position"
    );
}
