//! Embedding one dataset's corpus and scoring its queries against it.

use crate::dataset::{Document, Query};
use crate::gpu::{self, Sample};
use crate::metrics::Report;
use std::io::Write;
use std::time::{Duration, Instant};
use telividb_core::{Dim, Metric};
use telividb_embed::{GgmlInferencer, Inferencer, ModelId, Task};
use telividb_index::VectorIndex;
use telividb_index::adapters::{GpuFlatIndex, MemoryStore};

/// Corpus documents handed to the inference server at once.
///
/// Not a batch size: the server batches internally by token budget, sorting by
/// length so a long document does not pad short ones up to its size. This is
/// only how much is held in memory at a time before being pushed to the store.
const CHUNK: usize = 512;

/// How deep to retrieve. Recall@100 is reported, so nothing shallower will do.
const TOP_K: usize = 100;

/// What one dataset's run measured.
pub struct Outcome {
    /// Which dataset this row is for, so a table can label it.
    pub name: String,
    /// Averaged retrieval metrics.
    pub report: Report,
    /// Documents embedded, which is the denominator of the throughput
    /// column.
    pub documents: usize,
    /// Where search ran: `metal`, `cuda` or `cpu`.
    pub device: &'static str,
    /// Time spent embedding the corpus.
    pub embed_time: Duration,
    /// Time spent embedding and searching every query.
    pub search_time: Duration,
    /// Device memory after it finished and its index was dropped.
    pub after: Sample,
}

impl Outcome {
    /// Documents embedded per second.
    pub fn docs_per_second(&self) -> f64 {
        self.documents as f64 / self.embed_time.as_secs_f64().max(1e-9)
    }

    /// Milliseconds to encode and answer one query.
    pub fn ms_per_query(&self) -> f64 {
        self.search_time.as_secs_f64() * 1000.0 / (self.report.queries.max(1) as f64)
    }
}

/// Run one dataset end to end.
pub fn evaluate(
    server: &GgmlInferencer,
    id: &ModelId,
    dim: Dim,
    name: &str,
    documents: &[Document],
    queries: &[Query],
) -> Result<Outcome, Box<dyn std::error::Error>> {
    // ---- Embed the corpus. ----
    let started = Instant::now();
    let mut store = MemoryStore::new(dim, Metric::Dot);
    let mut ids = Vec::with_capacity(documents.len());

    for chunk in documents.chunks(CHUNK) {
        let texts: Vec<String> = chunk.iter().map(|d| d.text.clone()).collect();
        for (document, vector) in chunk.iter().zip(server.embed(id, Task::Document, &texts)?) {
            store.push(&vector)?;
            ids.push(document.id.clone());
        }
        print!("\r  {name}: embedding {}/{}   ", ids.len(), documents.len());
        let _ = std::io::stdout().flush();
    }
    let embed_time = started.elapsed();

    // ---- Index exhaustively, so this measures the encoder. ----
    let index = GpuFlatIndex::build(&store)?;
    let device = index.device();

    // ---- Score every query. ----
    let started = Instant::now();
    let mut report = Report::default();
    for (done, query) in queries.iter().enumerate() {
        // Encoded as a *query*: the task prefix differs from a document's, and
        // the wrong one lowers recall while returning normal vectors.
        let encoded = server.embed(id, Task::Query, std::slice::from_ref(&query.text))?;
        let hits = index.search(&store, &encoded[0], TOP_K, None)?;

        // The query's own document is dropped before scoring.
        //
        // In ArguAna a query *is* a corpus document — 1,298 of its 1,406 test
        // queries are — and no qrel ever marks a query relevant to itself. So
        // the nearest neighbour is a near-exact self-match that is guaranteed
        // to be wrong, and leaving it in spends rank 1 on it for 92% of
        // queries. BEIR's reference evaluation excludes it, and a number
        // measured without this exclusion is not comparable to a published
        // one. Harmless where it does not apply: no other dataset here shares
        // ids between queries and corpus.
        let retrieved: Vec<String> = hits
            .iter()
            .map(|hit| ids[hit.ordinal.row() as usize].clone())
            .filter(|id| *id != query.id)
            .collect();
        report.add(&retrieved, &query.relevant);

        if done % 100 == 0 {
            print!("\r  {name}: scoring {done}/{}   ", queries.len());
            let _ = std::io::stdout().flush();
        }
    }
    let search_time = started.elapsed();

    // Dropped before sampling, so what remains is what was *not* released —
    // which is the only thing a leak check can be about.
    drop(index);
    drop(store);
    let after = gpu::sample();

    print!("\r{}\r", " ".repeat(48));
    Ok(Outcome {
        name: name.to_owned(),
        report: report.finish(),
        documents: documents.len(),
        device,
        embed_time,
        search_time,
        after,
    })
}
