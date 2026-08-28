//! The index half of the walkthrough.
//!
//! Split from `main.rs` so that file reads as the pipeline it describes —
//! load, embed, index, search — with the GPU-or-CPU selection handled here
//! rather than interrupting it.

use telividb_core::{Dim, Metric, Result};
use telividb_index::VectorIndex;
use telividb_index::adapters::{FlatIndex, MemoryStore};
use telividb_index::domain::Candidate;

/// An indexed corpus, ready to answer queries.
pub struct Corpus {
    store: MemoryStore,
    index: Box<dyn VectorIndex>,
    device: String,
}

impl Corpus {
    /// Load `vectors` into a store and index them.
    ///
    /// The GPU index is exhaustive — one matmul over every row — so results
    /// are exact and there is no build step and no recall number to report.
    /// That is the right trade for a corpus this size; HNSW earns its keep at
    /// a scale where scanning everything stops being free.
    pub fn build(vectors: &[Vec<f32>], dim: Dim, metric: Metric) -> Result<Self> {
        let mut store = MemoryStore::new(dim, metric);
        for vector in vectors {
            store.push(vector)?;
        }

        let (index, device) = select(&store)?;
        Ok(Self {
            store,
            index,
            device,
        })
    }

    /// Where the search actually runs.
    ///
    /// Printed by the walkthrough because an index that quietly fell back to
    /// CPU returns identical results and is otherwise invisible.
    pub fn device(&self) -> &str {
        &self.device
    }

    /// The `k` best matches for `query`, best first.
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<Candidate>> {
        // `None` for the visibility bitmap: this example has no principal and
        // no policy. A served query never passes `None` — the predicate is
        // ANDed in *before* the index runs (invariant 15), never applied to
        // the results afterwards.
        self.index.search(&self.store, query, k, None)
    }
}

/// Build the GPU index when the feature is on, falling back to flat.
#[cfg(feature = "gpu")]
fn select(store: &MemoryStore) -> Result<(Box<dyn VectorIndex>, String)> {
    use telividb_index::adapters::{Device, GpuFlatIndex};

    match GpuFlatIndex::build(store) {
        Ok(index) => {
            let device = index.device().to_owned();
            Ok((Box::new(index), device))
        }
        // A refusal here is a budget or a device problem, not a corpus
        // problem: the flat index answers the same queries with the same
        // results. Reported rather than swallowed, so a silent fallback is
        // still a visible one.
        Err(error) => {
            eprintln!("note: GPU index unavailable ({error}); using the flat index.");
            let device = Device::best().kind().as_str();
            Ok((
                Box::new(FlatIndex::default()),
                format!("{device} (flat fallback)"),
            ))
        }
    }
}

/// Without the `gpu` feature there is one index and no device question.
#[cfg(not(feature = "gpu"))]
fn select(_store: &MemoryStore) -> Result<(Box<dyn VectorIndex>, String)> {
    Ok((Box::new(FlatIndex::default()), "cpu (flat)".to_owned()))
}
