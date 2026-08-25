//! Recall versus throughput, on the datasets the field compares on.
//!
//! ```text
//! examples/datasets/download.sh siftsmall
//! cargo run --release -p telividb-examples --bin ann
//!
//! examples/datasets/download.sh sift          # 168 MB, 1M vectors
//! cargo run --release -p telividb-examples --bin ann -- sift
//! TELIVIDB_QUERIES=10000 cargo run --release -p telividb-examples --bin ann -- sift
//! ```
//!
//! Queries are capped at 1,000 by default. The exhaustive baselines cost
//! `queries x rows`, so on SIFT-1M the full 10,000 would take the flat index
//! about ten minutes — and every configuration has to answer the *same*
//! queries for the comparison to hold. A thousand is well past what recall@10
//! needs to be stable.
//!
//! **What this measures that `beir` does not.** The BEIR benchmark measures
//! the *encoder*: whether our implementation of a published model reproduces
//! its published accuracy. Any system running the same model would score the
//! same. This one measures the *index* — the recall-versus-QPS trade that
//! `ann-benchmarks` plots and that every ANN library is compared on.
//!
//! Ground truth here is exhaustive and shipped with the dataset, so recall is
//! exact rather than measured against another approximation.
//!
//! Output includes mermaid charts, so a result can be pasted into a README or
//! an issue without a plotting toolchain anywhere in the build.

mod chart;
mod report;
mod sweep;
mod vecs;

use std::time::Instant;
use sweep::Point;
use telividb_index::adapters::{FlatIndex, HnswIndex, HnswParams, IvfFlatIndex, IvfParams};

/// Search breadths to sweep.
///
/// `ef_search` is HNSW's accuracy dial: it bounds how many candidates a query
/// keeps in flight, so recall rises and throughput falls with it. Sweeping it
/// is what produces a curve rather than a single point — and a single point is
/// not comparable to anything, because another index at another setting can
/// always beat it on one axis.
const EF_SWEEP: &[usize] = &[10, 16, 32, 64, 128, 256];

/// Fractions of `nlist` to probe.
///
/// Expressed as fractions rather than absolute counts because `nlist` scales
/// with the corpus — probing 8 lists means something different at 256 lists
/// than at 1,000, and a curve has to compare like with like.
const PROBE_SWEEP: &[f64] = &[0.005, 0.01, 0.02, 0.05, 0.10, 0.25];

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "siftsmall".to_owned());

    let dataset = match vecs::load(&name) {
        Ok(dataset) => dataset,
        Err(explanation) => {
            eprintln!("{explanation}");
            std::process::exit(1);
        }
    };
    println!(
        "{name}: {} base vectors x {} dims, {} queries",
        dataset.base.len(),
        dataset.dim,
        dataset.queries.len()
    );

    let store = match sweep::store_of(&dataset) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("could not load the corpus: {e}");
            std::process::exit(1);
        }
    };

    let mut points: Vec<Point> = Vec::new();

    // The exhaustive reference. Recall 1.0 by construction, so it is the
    // correctness check for everything below and the throughput floor.
    println!("  measuring flat (exhaustive)...");
    let flat = FlatIndex::new();
    match sweep::measure("flat", "flat", &dataset, &store, &flat, Default::default()) {
        Ok(point) => points.push(point),
        Err(e) => eprintln!("  flat failed: {e}"),
    }

    #[cfg(feature = "gpu")]
    measure_gpu(&dataset, &store, &mut points);

    // One graph, swept. `ef_search` is a query-time dial, so rebuilding per
    // setting would measure the builder instead of the search.
    println!("  building hnsw...");
    let built = Instant::now();
    let mut hnsw = HnswIndex::build(&store, HnswParams::default());
    let build = built.elapsed();
    println!("  built in {:.2}s, sweeping ef...", build.as_secs_f64());

    for ef in EF_SWEEP {
        hnsw = hnsw.with_ef_search(*ef);
        match sweep::measure(
            &format!("hnsw ef={ef}"),
            "hnsw",
            &dataset,
            &store,
            &hnsw,
            build,
        ) {
            Ok(point) => points.push(point),
            Err(e) => eprintln!("  hnsw ef={ef} failed: {e}"),
        }
    }

    // IVF: one partition, swept on nprobe — the same shape as ef for HNSW.
    let ivf_params = IvfParams::for_rows(dataset.base.len());
    println!("  building ivf (nlist={})...", ivf_params.nlist);
    let built = Instant::now();
    match IvfFlatIndex::build(&store, ivf_params) {
        Ok(mut ivf) => {
            let build = built.elapsed();
            let sizes = ivf.list_sizes();
            let largest = sizes.iter().copied().max().unwrap_or(0);
            println!(
                "  built in {:.2}s, {} lists, largest holds {} ({:.1}% of the corpus)",
                build.as_secs_f64(),
                sizes.len(),
                largest,
                100.0 * largest as f64 / dataset.base.len().max(1) as f64,
            );

            // Deduplicated: at a small `nlist` two fractions round to the same
            // count, and repeating a configuration pads the curve with a point
            // that says nothing new.
            let mut probed: Vec<usize> = PROBE_SWEEP
                .iter()
                .map(|f| ((ivf_params.nlist as f64) * f).round().max(1.0) as usize)
                .collect();
            probed.dedup();

            for nprobe in probed {
                ivf = ivf.with_nprobe(nprobe);
                let label = format!("ivf nprobe={nprobe}");
                match sweep::measure(&label, "ivf", &dataset, &store, &ivf, build) {
                    Ok(point) => points.push(point),
                    Err(e) => eprintln!("  {label} failed: {e}"),
                }
            }
        }
        Err(e) => eprintln!("  ivf unavailable: {e}"),
    }

    report::print_table(&name, &points);
    report::print_charts(&name, &points);
    report::print_notes(&points);
}

/// Measure the GPU exhaustive index, when it is compiled in.
#[cfg(feature = "gpu")]
fn measure_gpu(
    dataset: &vecs::Dataset,
    store: &telividb_index::adapters::MemoryStore,
    points: &mut Vec<Point>,
) {
    use telividb_index::adapters::GpuFlatIndex;

    println!("  measuring gpu-flat (exhaustive)...");
    let built = Instant::now();
    match GpuFlatIndex::build(store) {
        Ok(index) => {
            let build = built.elapsed();
            let label = format!("gpu-flat ({})", index.device());
            match sweep::measure(&label, "gpu-flat", dataset, store, &index, build) {
                Ok(point) => points.push(point),
                Err(e) => eprintln!("  gpu-flat failed: {e}"),
            }
        }
        // Reported rather than swallowed: a silent fallback would make the
        // table look like the GPU was simply slow.
        Err(e) => eprintln!("  gpu-flat unavailable: {e}"),
    }
}
