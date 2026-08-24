//! Accuracy on a standard dataset, measured rather than eyeballed.
//!
//! ```text
//! examples/models/download.sh
//! examples/datasets/download.sh scifact
//! cargo run --release -p telividb-examples --bin scifact
//! ```
//!
//! **Why a real dataset.** The toy corpus in the other examples proves the
//! pipeline runs; it cannot prove it is *correct*. An encoder with a subtly
//! wrong tokenizer, pooling mode or rotation convention still returns
//! well-formed, unit-length, plausible vectors — and a twelve-sentence corpus
//! is far too easy to distinguish. Both bugs found while building this crate
//! were of exactly that kind.
//!
//! SciFact has 5,183 documents, 300 judged queries and graded relevance
//! judgements, so nDCG@10 here is directly comparable to the figure published
//! for this model. A broken encoder cannot fake it.
//!
//! Search is **exhaustive** (`GpuFlatIndex`), so this measures the *encoder*,
//! not an approximate index. An ANN recall number is a separate question,
//! answered by `cargo run -p telividb-index --bin recall`.

mod dataset;
mod metrics;
mod report;
mod run;

use telividb_examples::model;

/// Below this, something is broken rather than merely mediocre.
///
/// Deliberately well under the published figure: this is a *correctness*
/// floor, not a quality target. A healthy nomic-embed-text-v1.5 scores in the
/// low 0.7s on SciFact; the tokenizer bug found while building this crate
/// would have landed near 0.2. Anything under this and the pipeline is wrong
/// somewhere, which is the failure worth catching automatically.
const NDCG_FLOOR: f64 = 0.55;

fn main() {
    let model_path = match model::default_text_model() {
        Ok(path) => path,
        Err(explanation) => {
            eprintln!("{explanation}");
            std::process::exit(1);
        }
    };

    let (documents, queries) = match dataset::load("scifact") {
        Ok(loaded) => loaded,
        Err(explanation) => {
            eprintln!("{explanation}");
            std::process::exit(1);
        }
    };

    println!(
        "scifact: {} documents, {} judged queries",
        documents.len(),
        queries.len()
    );
    println!("model  : {}\n", model_path.display());

    let outcome = match run::evaluate(&model_path, &documents, &queries) {
        Ok(outcome) => outcome,
        Err(error) => {
            eprintln!("evaluation failed: {error}");
            std::process::exit(1);
        }
    };

    report::print(&outcome, NDCG_FLOOR);

    // A non-zero exit so this can gate a change rather than merely inform one.
    if outcome.report.ndcg_at_10 < NDCG_FLOOR {
        std::process::exit(1);
    }
}
