//! The walkthrough: real model, real text, real search.
//!
//! Download a GGUF embedding model, embed a small corpus with it, build a
//! GPU-resident index over the vectors, and answer questions against it —
//! then print what the process is holding.
//!
//! ```text
//! examples/models/download.sh
//! cargo run --release -p telividb-examples --bin semantic_search
//! ```
//!
//! **Read the rankings, not just the numbers.** Every query here has an
//! obviously correct answer. The failure this guards against is the one that
//! does not raise an error: a dropped task prefix, a misread pooling mode, a
//! mismatched rotation convention all produce well-formed, unit-length,
//! entirely plausible vectors. The only thing that exposes them is a ranking
//! that is visibly wrong.
//!
//! `--release` matters more than usual: a debug build runs the encoder's
//! matmuls unoptimized and takes minutes where release takes seconds.

mod search;

use telividb_core::{Fingerprint, Metric};
use telividb_embed::{GgmlInferencer, Inferencer, ModelId, Task};
use telividb_examples::{corpus, model, report};

fn main() {
    let path = match model::default_text_model() {
        Ok(path) => path,
        Err(explanation) => {
            eprintln!("{explanation}");
            std::process::exit(1);
        }
    };

    // ---- 1. Load the model, and keep it loaded. -------------------------
    //
    // The digest is computed from the file and becomes the model's identity
    // (rule 12). Passing `Fingerprint::unset()` means "whatever this file is";
    // a real deployment pins the digest its vector field was built with, so a
    // swapped file is refused rather than silently mixed in.
    let mut server = GgmlInferencer::new();
    let id = ModelId::new("nomic-embed-text-v1.5", Fingerprint::unset());

    println!("loading {}...", path.display());
    if let Err(error) = server.register(&id, &path) {
        eprintln!("could not load the model: {error}");
        std::process::exit(1);
    }

    let dim = server.dim(&id).expect("the model just loaded");
    println!("resident: {} ({} dimensions)\n", id, dim.get());

    // ---- 2. Embed the corpus. -------------------------------------------
    //
    // As *documents*. The task prefix is trained in, and embedding a corpus
    // with the query prefix measurably lowers recall while returning vectors
    // that look entirely normal.
    let documents: Vec<String> = corpus::DOCUMENTS.iter().map(|s| s.to_string()).collect();
    let vectors = match server.embed(&id, Task::Document, &documents) {
        Ok(vectors) => vectors,
        Err(error) => {
            eprintln!("could not embed the corpus: {error}");
            std::process::exit(1);
        }
    };
    println!("embedded {} documents.", vectors.len());

    // ---- 3. Index them. --------------------------------------------------
    //
    // Cosine similarity over unit-length vectors is a dot product, and the
    // inference server already normalized them — so the metric here is `Dot`
    // rather than a second normalization at search time.
    let index = match search::Corpus::build(&vectors, dim, Metric::Dot) {
        Ok(index) => index,
        Err(error) => {
            eprintln!("could not build the index: {error}");
            std::process::exit(1);
        }
    };
    println!("indexed on {}.\n", index.device());

    // ---- 4. Search. ------------------------------------------------------
    for query in corpus::QUERIES {
        // As a *query* this time — the other side of the asymmetry.
        let encoded = match server.embed(&id, Task::Query, &[query.to_string()]) {
            Ok(mut v) => v.remove(0),
            Err(error) => {
                eprintln!("could not embed the query: {error}");
                continue;
            }
        };

        println!("? {query}");
        match index.search(&encoded, 3) {
            Ok(hits) => {
                for (rank, hit) in hits.iter().enumerate() {
                    println!(
                        "  {}. {:.4}  {}",
                        rank + 1,
                        hit.score,
                        corpus::DOCUMENTS[hit.ordinal.row() as usize]
                    );
                }
            }
            Err(error) => eprintln!("  search failed: {error}"),
        }
        println!();
    }

    // ---- 5. What is this process holding? --------------------------------
    println!("---- residency ----");
    report::print_residency();

    println!(
        "\nNote the model dominates: {} of weights against a corpus of {} \
         documents.\nThat ratio is the whole reason models stay resident \
         (rule 45) — reloading\nper request would cost far more than the search \
         itself ever does.",
        report::mib(
            std::fs::metadata(&path)
                .map(|m| m.len() as usize)
                .unwrap_or(0)
        ),
        vectors.len(),
    );
}
