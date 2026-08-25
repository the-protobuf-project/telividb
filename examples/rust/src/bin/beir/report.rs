//! Printing the results, and saying what they mean.

use crate::gpu::{self, Sample};
use crate::run::Outcome;
use std::path::Path;

/// Print what is about to run.
pub fn print_header(model: &Path, dim: usize, baseline: &Sample) {
    println!("model      : {}", model.display());
    println!("dimensions : {dim}");
    print!(
        "device mem : {:.1} MiB reserved",
        gpu::mib(baseline.reserved)
    );
    match baseline.allocated {
        Some(bytes) => println!(", {:.1} MiB allocated (metal)", gpu::mib(bytes)),
        None => println!(" (no device figure — CPU or CUDA)"),
    }
    println!();
}

/// The accuracy and throughput table.
pub fn print_results(outcomes: &[Outcome]) {
    println!("---- accuracy ----\n");
    println!(
        "{:<10} {:>7} {:>7}  {:>8} {:>8} {:>8}",
        "dataset", "docs", "queries", "nDCG@10", "R@10", "R@100"
    );
    println!("{}", "-".repeat(56));
    for o in outcomes {
        println!(
            "{:<10} {:>7} {:>7}  {:>8.4} {:>8.4} {:>8.4}",
            o.name,
            o.documents,
            o.report.queries,
            o.report.ndcg_at_10,
            o.report.recall_at_10,
            o.report.recall_at_100,
        );
    }

    println!("\n---- throughput ----\n");
    println!(
        "{:<10} {:>7}  {:>9} {:>9}  {:>9} {:>8}",
        "dataset", "docs", "embed", "docs/s", "search", "ms/query"
    );
    println!("{}", "-".repeat(60));
    for o in outcomes {
        println!(
            "{:<10} {:>7}  {:>8.1}s {:>9.1}  {:>8.1}s {:>8.1}",
            o.name,
            o.documents,
            o.embed_time.as_secs_f64(),
            o.docs_per_second(),
            o.search_time.as_secs_f64(),
            o.ms_per_query(),
        );
    }
    println!("\n  device: {}", outcomes[0].device);
    println!("  Throughput is documents per second, not tokens: a corpus of long");
    println!("  documents scores lower on this column while doing more work.");
}

/// The leak check.
pub fn print_memory(baseline: &Sample, outcomes: &[Outcome]) {
    println!("\n---- device memory ----\n");
    println!(
        "{:<10} {:>12} {:>12} {:>12} {:>12}",
        "after", "reserved", "Δreserved", "allocated", "Δallocated"
    );
    println!("{}", "-".repeat(62));

    let mut previous = *baseline;
    for o in outcomes {
        let (reserved, allocated) = o.after.growth_since(&previous);
        println!(
            "{:<10} {:>11.1}M {:>11.1}M {:>11} {:>11}",
            o.name,
            gpu::mib(o.after.reserved),
            gpu::mib_signed(reserved),
            o.after
                .allocated
                .map(|b| format!("{:.1}M", gpu::mib(b)))
                .unwrap_or_else(|| "-".to_owned()),
            allocated
                .map(|b| format!("{:+.1}M", gpu::mib_signed(b)))
                .unwrap_or_else(|| "-".to_owned()),
        );
        previous = o.after;
    }

    let (reserved_growth, allocated_growth) = previous.growth_since(baseline);
    println!("\n  resident now:");
    for line in gpu::resident_lines() {
        println!("    {line}");
    }

    println!();
    interpret(reserved_growth, allocated_growth, &previous);
}

/// Say whether the numbers above indicate a leak.
///
/// Reserved growth is the registry's own accounting: each dataset's index is
/// dropped before the sample, so it should return to the baseline. Allocated
/// growth is what Metal reports device-wide, which does *not* have to return
/// to zero — the driver caches buffers rather than releasing them eagerly, and
/// a modest steady figure is normal.
///
/// A leak is reserved growth that never comes back, or allocated growth that
/// keeps rising in step with the corpora rather than levelling off.
fn interpret(reserved: i64, allocated: Option<i64>, last: &Sample) {
    // One index and one model may legitimately remain; more means a handle
    // was not dropped.
    if reserved.abs() > 1024 * 1024 {
        println!(
            "  LEAK? reserved grew {:+.1} MiB across the run and did not return.",
            gpu::mib_signed(reserved)
        );
        println!("  Every index is dropped before its sample, so this should be ~0.");
        println!("  Check for a `residency::Handle` held past its owner's lifetime.");
    } else {
        println!(
            "  Reserved returned to baseline ({:+.1} MiB): every index released.",
            gpu::mib_signed(reserved)
        );
    }

    match allocated {
        Some(growth) if growth > 256 * 1024 * 1024 => {
            println!(
                "  LEAK? metal still holds {:+.1} MiB more than at the start.",
                gpu::mib_signed(growth)
            );
            println!("  Some is the driver's buffer cache, but this much suggests");
            println!("  tensors are being retained — look for a Vec of intermediates");
            println!("  that outlives a batch.");
        }
        Some(growth) => {
            println!(
                "  Metal holds {:+.1} MiB more than at the start, which is its",
                gpu::mib_signed(growth)
            );
            println!("  buffer cache rather than a leak — it does not release eagerly.");
        }
        None => println!("  No device figure on this platform; only reserved was checked."),
    }

    println!(
        "\n  {} model(s) and {} index(es) still resident.",
        last.models, last.indexes
    );
}
