//! Where a device-resident search actually spends its time.
//!
//! ```text
//! cargo run --release -p telividb-examples --bin gpu_profile
//! cargo run --release -p telividb-examples --bin gpu_profile -- 1000000 128
//! cargo run --release -p telividb-examples --bin gpu_profile -- 1000000 128 l2
//! ```
//!
//! The metric is an argument because it is not merely a scoring detail: L2
//! carries a per-row norm the other metrics do not, computed once at build
//! time. A profile that only ever ran one metric would never see that cost.
//!
//! Two questions, both of which turn out to have surprising answers:
//!
//! 1. **What does building cost?** The corpus is rebuilt on every load rather
//!    than persisted (CLAUDE.md rule 46), so build time is paid at startup for
//!    real. A number that is wrong by two orders of magnitude turns a design
//!    decision into a bad one, which is why this measures it rather than
//!    quoting it.
//!
//! 2. **Which half of a query dominates?** A batched query is a device matmul
//!    followed by a host top-k pass, and "the device scores; the host decides"
//!    is only a good split while the host half stays small. Running the same
//!    batch sizes under `RAYON_NUM_THREADS=1` and then unconstrained isolates
//!    it: parallel selection is the *only* thing that varies, so the difference
//!    between the two runs is the selection half's share.
//!
//!    ```text
//!    RAYON_NUM_THREADS=1 cargo run --release -p telividb-examples --bin gpu_profile
//!    ```
//!
//! **Synthetic vectors, deliberately.** This measures time, not recall, and an
//! exhaustive index scores every row whatever the distribution — so a generated
//! corpus measures exactly what SIFT would while needing no download. Recall
//! lives in `ann`, against real datasets and real ground truth.
#![forbid(unsafe_code)]

use std::time::Instant;
use telividb_core::{Dim, Metric, VectorStore};
use telividb_index::adapters::MemoryStore;
use telividb_index::ports::VectorIndex;

/// Batch sizes to report, spanning one query to past the internal chunk size.
const BATCHES: &[usize] = &[1, 8, 32, 64];

/// Queries answered per batch size. Enough to average out a cold first call.
const REPEATS: usize = 20;

fn main() {
    let mut args = std::env::args().skip(1);
    let rows: usize = args
        .next()
        .and_then(|a| a.parse().ok())
        .unwrap_or(1_000_000);
    let dim: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(128);
    let metric = match args.next().as_deref() {
        Some("l2") => Metric::L2,
        Some("cosine") => Metric::Cosine,
        _ => Metric::Dot,
    };

    println!("corpus: {rows} rows x {dim} dims, metric={metric:?}");
    let filling = Instant::now();
    let store = fill(rows, dim, metric);
    println!(
        "\nhost store filled in {:.2}s",
        filling.elapsed().as_secs_f64()
    );

    #[cfg(feature = "gpu")]
    profile(&store, dim);

    #[cfg(not(feature = "gpu"))]
    {
        let _ = (&store, dim);
        eprintln!("built without the `gpu` feature; nothing to profile.");
    }
}

/// A corpus of distinct, non-degenerate vectors.
fn fill(rows: usize, dim: usize, metric: Metric) -> MemoryStore {
    let mut store = MemoryStore::new(Dim::new(dim as u32).unwrap(), metric);
    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut vector = vec![0f32; dim];
    for _ in 0..rows {
        for slot in vector.iter_mut() {
            // xorshift: cheap, deterministic, and good enough for timing.
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            *slot = (seed >> 40) as f32 / 16_777_216.0;
        }
        store.push(&vector).unwrap();
    }
    store
}

#[cfg(feature = "gpu")]
fn profile(store: &MemoryStore, dim: usize) {
    use telividb_index::adapters::GpuFlatIndex;

    let building = Instant::now();
    let index = match GpuFlatIndex::build(store) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("could not build the device index: {e}");
            return;
        }
    };
    let build = building.elapsed().as_secs_f64();
    println!("device      : {}", index.device());
    println!("selection   : {}", GpuFlatIndex::selection());
    println!("build       : {build:.3}s\n");

    println!("  batch   per-query   queries/s");
    println!("  -----   ---------   ---------");
    for size in BATCHES {
        let queries = queries_of(*size, dim);
        let refs: Vec<&[f32]> = queries.iter().map(|q| q.as_slice()).collect();

        // One untimed call: the first touches cold buffers and would otherwise
        // be charged to the smallest batch size.
        let _ = index.search_batch(store, &refs, 10, None);

        let started = Instant::now();
        for _ in 0..REPEATS {
            if let Err(e) = index.search_batch(store, &refs, 10, None) {
                eprintln!("  batch {size} failed: {e}");
                return;
            }
        }
        let elapsed = started.elapsed().as_secs_f64();
        let answered = (REPEATS * size) as f64;
        println!(
            "  {size:>5}   {:>7.3}ms   {:>9.0}",
            elapsed / answered * 1000.0,
            answered / elapsed,
        );
    }

    println!(
        "\nrows scored: {} per query, {} bytes of scores per batch of 32",
        store.len(),
        store.len() * 32 * 4
    );
}

/// Distinct query vectors, unrelated to the corpus so nothing scores trivially.
#[cfg(feature = "gpu")]
fn queries_of(count: usize, dim: usize) -> Vec<Vec<f32>> {
    (0..count)
        .map(|q| {
            (0..dim)
                .map(|d| ((q * 31 + d * 17) % 97) as f32 / 97.0)
                .collect()
        })
        .collect()
}
