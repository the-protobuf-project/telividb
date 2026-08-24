//! Recall harness.
//!
//! Builds a synthetic corpus, indexes it, and reports recall@k against
//! exhaustive search. Use it to see what `ef_search` buys before changing a
//! default, and to check that a change to the graph did not cost quality.
//!
//! ```text
//! cargo run --release -p telividb-index --bin recall -- --rows 100000 --ef 64
//! ```
//!
//! Synthetic data only. Real benchmark sets — SIFT, GIST, Deep1B — arrive with
//! the `fvecs` reader in the bulk I/O phase; until then this measures the graph
//! rather than the data distribution, which is what a regression check needs.

mod args;

use args::{Distribution, parse};
use std::time::Instant;
use telividb_core::Dim;
use telividb_index::{
    FlatIndex, HnswIndex, HnswParams, RecallReport, VectorIndex, adapters::MemoryStore, recall_at_k,
};

struct Rng(u64);

impl Rng {
    /// Approximately standard normal, by summing uniforms.
    fn next_gaussian(&mut self) -> f32 {
        (0..6).map(|_| self.next_f32()).sum::<f32>() / 6f32.sqrt()
    }

    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 40) as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }
}

fn main() {
    let args = parse();
    let mut rng = Rng(0xC0FF_EE00_1234_5678);

    println!(
        "corpus: {} rows x {} dims, metric {:?}, {:?}",
        args.rows, args.dim, args.metric, args.distribution
    );

    let mut store = MemoryStore::new(Dim::new(args.dim as u32).unwrap(), args.metric);
    let centres: Vec<Vec<f32>> = (0..args.clusters)
        .map(|_| (0..args.dim).map(|_| rng.next_f32()).collect())
        .collect();

    for row in 0..args.rows {
        let v: Vec<f32> = match args.distribution {
            Distribution::Uniform => (0..args.dim).map(|_| rng.next_f32()).collect(),
            Distribution::Clustered => {
                let centre = &centres[row % args.clusters];
                centre
                    .iter()
                    .map(|c| c + rng.next_gaussian() * 0.15)
                    .collect()
            }
        };
        store.push(&v).expect("generated vector is valid");
    }

    let started = Instant::now();
    let hnsw = HnswIndex::build(
        &store,
        HnswParams {
            ef_search: args.ef,
            batch_size: args.batch,
            ..Default::default()
        },
    );
    let build = started.elapsed();
    println!(
        "build:  {:.2}s, {} edges across {} levels",
        build.as_secs_f64(),
        hnsw.graph().edge_count(),
        hnsw.graph().max_level() + 1
    );

    // Queries are drawn from the same distribution as the corpus. Querying a
    // clustered corpus with uniform noise measures nothing anyone will ever do.
    let queries: Vec<Vec<f32>> = (0..args.queries)
        .map(|i| match args.distribution {
            Distribution::Uniform => (0..args.dim).map(|_| rng.next_f32()).collect(),
            Distribution::Clustered => centres[i % args.clusters]
                .iter()
                .map(|c| c + rng.next_gaussian() * 0.15)
                .collect(),
        })
        .collect();

    let mut per_query = Vec::with_capacity(queries.len());
    let mut approx_time = std::time::Duration::ZERO;
    let mut exact_time = std::time::Duration::ZERO;

    for q in &queries {
        let t = Instant::now();
        let truth = FlatIndex::new()
            .search(&store, q, args.k, None)
            .expect("flat search");
        exact_time += t.elapsed();

        let t = Instant::now();
        let approx = hnsw.search(&store, q, args.k, None).expect("hnsw search");
        approx_time += t.elapsed();

        per_query.push(recall_at_k(&approx, &truth, args.k));
    }

    let report = RecallReport::of(&per_query, args.k);
    let n = args.queries as u32;
    println!("hnsw:   {:?} per query (ef {})", approx_time / n, args.ef);
    println!("flat:   {:?} per query", exact_time / n);
    println!(
        "speedup: {:.1}x",
        exact_time.as_secs_f64() / approx_time.as_secs_f64().max(f64::EPSILON)
    );
    println!("{report}");

    // Non-zero exit on a bad result, so this is usable as a gate.
    if !report.meets(0.95) {
        eprintln!("FAIL: mean recall below 0.95");
        std::process::exit(1);
    }
}
