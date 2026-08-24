//! One corpus size, taken through the whole create/read/update/delete cycle.

use std::time::Instant;
use telividb_core::{Dim, Metric};
use telividb_index::VectorIndex;
use telividb_index::adapters::{FlatIndex, GpuFlatIndex, MemoryStore};

/// What to measure, and how far to scale.
pub struct Args {
    /// Vector width. 768 is bge/e5-large's, the shape a real collection has.
    pub dim: usize,
    /// Stop once a scale would exceed this many rows.
    pub max_rows: usize,
    /// Neighbours per query.
    pub k: usize,
    /// Queries to average the read timing over.
    pub queries: usize,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            dim: 768,
            max_rows: 500_000,
            k: 10,
            queries: 20,
        }
    }
}

/// Deterministic, dependency-free vectors — the same SplitMix64 the test
/// fixtures use, so a surprising result here is reproducible there.
struct Rng(u64);

impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 40) as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }

    fn vector(&mut self, dim: usize) -> Vec<f32> {
        (0..dim).map(|_| self.next_f32()).collect()
    }
}

fn corpus(rows: usize, dim: usize, seed: u64) -> MemoryStore {
    let mut rng = Rng(seed);
    let mut store = MemoryStore::new(Dim::new(dim as u32).unwrap(), Metric::Dot);
    for _ in 0..rows {
        store.push(&rng.vector(dim)).unwrap();
    }
    store
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// One corpus size, through the whole CRUD cycle.
pub fn run_scale(args: &Args, rows: usize) {
    let resident = rows * args.dim * 4;
    let store = corpus(rows, args.dim, 0xC0FFEE);

    // CREATE — build the GGUF and upload it to the device.
    let started = Instant::now();
    let index = match GpuFlatIndex::build(&store) {
        Ok(index) => index,
        Err(e) => {
            // Expected once the corpus passes the budget: this is the guard
            // doing its job, not a crash. Before it existed, this scale killed
            // the process outright.
            println!("{rows:>10}  {:>9.1}M  refused: {e}", mib(resident));
            return;
        }
    };
    let create = started.elapsed().as_secs_f64();

    // READ — search, which is where the matmul actually runs.
    let mut rng = Rng(0xD15EA5E);
    let started = Instant::now();
    for _ in 0..args.queries {
        let query = rng.vector(args.dim);
        if let Err(e) = index.search(&store, &query, args.k, None) {
            println!("{rows:>10}  {:>9.1}M  refused at read: {e}", mib(resident));
            return;
        }
    }
    let read = started.elapsed().as_secs_f64() / args.queries as f64;

    // UPDATE — the corpus is immutable once uploaded, exactly as a sealed
    // segment is, so an update is a rebuild. This is the step that transiently
    // holds two copies on the device, and therefore the one that breaks first.
    let started = Instant::now();
    let updated = GpuFlatIndex::build(&store);
    let update = started.elapsed().as_secs_f64();
    if let Err(e) = updated {
        println!(
            "{rows:>10}  {:>9.1}M  refused at update: {e}",
            mib(resident)
        );
        return;
    }

    // Correctness has to travel with the timings: "it got faster" without a
    // correctness number is not a result (invariant 8). At scale this is the
    // check that a big upload did not silently truncate or misalign.
    let exact = agrees_with_flat(&store, &index, &rng.vector(args.dim), args.k);

    // DELETE — dropping the index must release the device allocation. If it
    // does not, the next scale up is what discovers it.
    drop(updated);
    drop(index);

    println!(
        "{rows:>10}  {:>9.1}M  {create:>9.3}s  {:>9.3}ms  {update:>9.3}s  {:>8}",
        mib(resident),
        read * 1000.0,
        if exact { "yes" } else { "NO" },
    );
}

/// Spot-check that scale has not cost correctness.
///
/// Compares one query against exhaustive CPU search. Scores rather than
/// ordinals, because a fused device matmul and a scalar CPU loop break exact
/// ties differently — see `tests/gpu_recall.rs` for the measured case.
fn agrees_with_flat(store: &MemoryStore, index: &GpuFlatIndex, query: &[f32], k: usize) -> bool {
    let flat = FlatIndex::new();
    let Ok(expected) = flat.search(store, query, k, None) else {
        return false;
    };
    let Ok(actual) = index.search(store, query, k, None) else {
        return false;
    };
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual.iter())
            .all(|(a, b)| (a.score - b.score).abs() <= 1e-4)
}
