//! Accuracy and GPU throughput across several BEIR datasets.
//!
//! ```text
//! examples/models/download.sh
//! examples/datasets/download.sh
//! cargo run --release -p telividb-examples --bin beir
//! cargo run --release -p telividb-examples --bin beir -- scifact nfcorpus
//! TELIVIDB_MAX_TOKENS=512 cargo run --release -p telividb-examples --bin beir
//! ```
//!
//! `TELIVIDB_MAX_TOKENS` caps sequence length below the model's context.
//! Attention is quadratic in that length, so the cap is the single biggest
//! throughput lever — and BEIR's own evaluations cap BERT-family models at
//! 512. Whether it costs accuracy is a question for the corpus, which is why
//! it is a knob here rather than a default.
//!
//! **Why a real dataset, and why several.** A toy corpus proves the pipeline
//! runs; it cannot prove it is correct. An encoder with a subtly wrong
//! tokenizer, pooling mode or rotation convention still returns well-formed,
//! unit-length, plausible vectors — both bugs found while building this crate
//! were exactly that. Graded relevance judgements make nDCG@10 comparable to
//! published figures, which a twelve-sentence corpus never could be.
//!
//! Several, because one corpus size cannot show whether cost scales with the
//! corpus or with something else. These span 3.6k to 57.6k documents.
//!
//! Search is **exhaustive** (`GpuFlatIndex`), so this measures the encoder
//! rather than an approximate index. ANN recall is a separate question,
//! answered by `cargo run -p telividb-index --bin recall`.

mod dataset;
mod gpu;
mod metrics;
mod report;
mod run;

use telividb_core::Fingerprint;
use telividb_embed::{CandleInferencer, Inferencer, ModelId};
use telividb_examples::model;

/// Below this, something is broken rather than merely mediocre.
///
/// A *correctness* floor, not a quality target, and deliberately well under
/// what a healthy model scores. The tokenizer bug found while building this
/// crate would have landed near 0.2; anything under this means the pipeline is
/// wrong somewhere, which is the failure worth catching automatically.
///
/// Per dataset, because they are not equally easy — ArguAna in particular is
/// an argument-retrieval task where the query *is* a passage.
const NDCG_FLOOR: f64 = 0.20;

fn main() {
    let model_path = match model::default_text_model() {
        Ok(path) => path,
        Err(explanation) => {
            eprintln!("{explanation}");
            std::process::exit(1);
        }
    };

    // Which datasets: whatever was named, else every one.
    let requested: Vec<String> = std::env::args().skip(1).collect();
    let names: Vec<String> = match requested.is_empty() {
        true => dataset::ALL.iter().map(|s| (*s).to_owned()).collect(),
        false => requested,
    };

    // Loaded once and held resident for every dataset (rule 45). Reloading per
    // dataset would dominate the timings and measure the loader instead.
    let mut server = match max_tokens() {
        Some(cap) => {
            println!("max tokens : {cap} (capped below the model's context)");
            CandleInferencer::new().with_max_tokens(cap)
        }
        None => CandleInferencer::new(),
    };
    let id = ModelId::new("beir-eval", Fingerprint::unset());
    if let Err(e) = server.register(&id, &model_path) {
        eprintln!("could not load the model: {e}");
        std::process::exit(1);
    }
    let dim = server.dim(&id).expect("the model just loaded");

    let baseline = gpu::sample();
    report::print_header(&model_path, dim.get(), &baseline);

    let mut outcomes = Vec::new();
    for name in &names {
        let (documents, queries) = match dataset::load(name) {
            Ok(loaded) => loaded,
            Err(explanation) => {
                eprintln!("  {name}: skipped — {explanation}");
                continue;
            }
        };

        match run::evaluate(&server, &id, dim, name, &documents, &queries) {
            Ok(outcome) => outcomes.push(outcome),
            Err(e) => eprintln!("  {name}: failed — {e}"),
        }
    }

    if outcomes.is_empty() {
        eprintln!("nothing ran; fetch the datasets with examples/datasets/download.sh");
        std::process::exit(1);
    }

    report::print_results(&outcomes);
    report::print_memory(&baseline, &outcomes);

    // A non-zero exit so this can gate a change rather than merely inform one.
    let broken: Vec<&str> = outcomes
        .iter()
        .filter(|o| o.report.ndcg_at_10 < NDCG_FLOOR)
        .map(|o| o.name.as_str())
        .collect();
    if !broken.is_empty() {
        eprintln!("\nFAIL — below the {NDCG_FLOOR:.2} floor: {broken:?}");
        std::process::exit(1);
    }
}

/// The sequence-length cap, if the environment sets one.
///
/// An environment variable rather than a flag because the positional arguments
/// are already dataset names, and a run's identity is the datasets it covered.
fn max_tokens() -> Option<usize> {
    std::env::var("TELIVIDB_MAX_TOKENS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .filter(|cap| *cap > 0)
}
