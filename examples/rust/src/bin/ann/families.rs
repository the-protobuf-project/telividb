//! Sweeping the partitioned index families.
//!
//! Split from `main.rs` so that file reads as the sequence of families a run
//! covers, rather than the details of each one's parameters.

use crate::sweep::{self, Point};
use crate::vecs::Dataset;
use std::time::Instant;
use telividb_distance::pq::PqParams;
use telividb_index::adapters::{IvfFlatIndex, IvfParams, IvfPqIndex, MemoryStore};

/// Fractions of `nlist` to probe.
///
/// Fractions rather than absolute counts because `nlist` scales with the
/// corpus — probing 8 lists means something different at 256 lists than at
/// 1,000, and a curve has to compare like with like.
const PROBE_SWEEP: &[f64] = &[0.005, 0.01, 0.02, 0.05, 0.10, 0.25];

/// Build and sweep both IVF families.
pub(crate) fn sweep_ivf(dataset: &Dataset, store: &MemoryStore, points: &mut Vec<Point>) {
    // IVF: one partition, swept on nprobe — the same shape as ef for HNSW.
    let ivf_params = IvfParams::for_rows(dataset.base.len());
    println!("  building ivf (nlist={})...", ivf_params.nlist);
    let built = Instant::now();
    match IvfFlatIndex::build(store, ivf_params) {
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
                match sweep::measure(&label, "ivf", dataset, store, &ivf, build) {
                    Ok(point) => points.push(point),
                    Err(e) => eprintln!("  {label} failed: {e}"),
                }
            }
        }
        Err(e) => eprintln!("  ivf unavailable: {e}"),
    }

    // IVF-PQ: the same partition, with each row stored as `m` bytes instead
    // of a full vector. Its column in the table is bytes per row, which is the
    // point of it.
    println!("  building ivf-pq...");
    let built = Instant::now();
    match IvfPqIndex::build(store, ivf_params, PqParams::default()) {
        Ok(mut pq) => {
            let build = built.elapsed();
            println!(
                "  built in {:.2}s, {} bytes per row (vs {} as f32)",
                build.as_secs_f64(),
                pq.bytes_per_row(),
                dataset.dim * 4,
            );
            for nprobe in [4usize, 16, 64] {
                let nprobe = nprobe.min(ivf_params.nlist);
                pq = pq.with_nprobe(nprobe);
                let label = format!("ivf-pq nprobe={nprobe}");
                match sweep::measure(&label, "ivf-pq", dataset, store, &pq, build) {
                    Ok(point) => points.push(point),
                    Err(e) => eprintln!("  {label} failed: {e}"),
                }
            }
        }
        Err(e) => eprintln!("  ivf-pq unavailable: {e}"),
    }
}
