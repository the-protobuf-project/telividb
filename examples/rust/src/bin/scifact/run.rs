//! Embedding the corpus and scoring every query against it.

use crate::dataset::{Document, Query};
use crate::metrics::Report;
use std::path::Path;
use std::time::{Duration, Instant};
use telividb_core::{Fingerprint, Metric};
use telividb_embed::{CandleInferencer, Inferencer, ModelId, Task};
use telividb_index::VectorIndex;
use telividb_index::adapters::{GpuFlatIndex, MemoryStore};

/// Texts per inference call.
///
/// Batched because padding to the batch's longest sequence is what keeps the
/// device busy; a per-document call leaves it idle between dispatches. Bounded
/// because a batch is padded to its longest member, so an unbounded one would
/// pad every short abstract out to the longest in the corpus.
const BATCH: usize = 32;

/// How deep to retrieve. Recall@100 is reported, so nothing shallower will do.
const TOP_K: usize = 100;

/// What the run measured, and what it cost.
pub struct Outcome {
    /// Averaged retrieval metrics.
    pub report: Report,
    /// Where the work ran: `metal`, `cuda` or `cpu`.
    pub device: &'static str,
    /// Vector width the model produced.
    pub dim: usize,
    /// Time spent embedding the corpus.
    pub embed_time: Duration,
    /// Time spent embedding and searching every query.
    pub search_time: Duration,
}

/// Embed the corpus, then score every query against it.
pub fn evaluate(
    model_path: &Path,
    documents: &[Document],
    queries: &[Query],
) -> Result<Outcome, Box<dyn std::error::Error>> {
    let mut server = CandleInferencer::new();
    let id = ModelId::new("scifact-eval", Fingerprint::unset());
    server.register(&id, model_path)?;
    let dim = server.dim(&id)?;

    // ---- Embed the corpus. ----
    let started = Instant::now();
    let mut store = MemoryStore::new(dim, Metric::Dot);
    let mut ids = Vec::with_capacity(documents.len());

    for (done, chunk) in documents.chunks(BATCH).enumerate() {
        let texts: Vec<String> = chunk.iter().map(|d| d.text.clone()).collect();
        for (document, vector) in chunk.iter().zip(server.embed(&id, Task::Document, &texts)?) {
            store.push(&vector)?;
            ids.push(document.id.clone());
        }
        if done % 20 == 0 {
            let seen = (done * BATCH).min(documents.len());
            print!("\r  embedding corpus: {seen}/{}", documents.len());
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }
    let embed_time = started.elapsed();
    println!(
        "\r  embedded {} documents in {:.1}s      ",
        documents.len(),
        embed_time.as_secs_f64()
    );

    // ---- Index exhaustively. ----
    //
    // Exact by construction, so what follows measures the encoder rather than
    // an approximation of it.
    let index = GpuFlatIndex::build(&store)?;
    let device = index.device();
    println!("  indexed on {device}");

    // ---- Score every query. ----
    let started = Instant::now();
    let mut report = Report::default();
    for (done, query) in queries.iter().enumerate() {
        // Encoded as a *query*, not a document: the task prefix differs, and
        // using the wrong one lowers recall while returning normal vectors.
        let encoded = server.embed(&id, Task::Query, std::slice::from_ref(&query.text))?;
        let hits = index.search(&store, &encoded[0], TOP_K, None)?;

        let retrieved: Vec<String> = hits
            .iter()
            .map(|hit| ids[hit.ordinal.row() as usize].clone())
            .collect();
        report.add(&retrieved, &query.relevant);

        if done % 50 == 0 {
            print!("\r  scoring queries: {done}/{}", queries.len());
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }
    let search_time = started.elapsed();
    println!(
        "\r  scored {} queries in {:.1}s      \n",
        queries.len(),
        search_time.as_secs_f64()
    );

    Ok(Outcome {
        report: report.finish(),
        device,
        dim: dim.get(),
        embed_time,
        search_time,
    })
}
