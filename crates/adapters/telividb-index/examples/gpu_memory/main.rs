//! Does the GPU index hold up as the corpus grows?
//!
//! Runs create / read / update / delete against `GpuFlatIndex` at increasing
//! corpus sizes, reporting the device, the resident bytes, and the time each
//! step takes. It exists because **VRAM budgeting is an open design question**
//! (ARCHITECTURE §15 Gap 22): nothing in the codebase yet decides what happens
//! when a corpus outgrows the device, so the first useful thing is to find out
//! where that point actually is on real hardware.
//!
//! ```text
//! cargo run --release --example gpu_memory
//! cargo run --release --example gpu_memory -- --dim 1536 --max-rows 2000000
//! ```
//!
//! Refusals are reported and the run continues: the interesting output is
//! *which* scale the budget stopped, and at what size.

mod scale;

use scale::{Args, run_scale};
use telividb_index::adapters::{
    BUDGET_ENV, best_device, budget_source, device_allocated_bytes, device_name, gpu_budget_bytes,
    gpu_resident_bytes,
};

/// First corpus size; each step doubles from here.
const FIRST_ROWS: usize = 10_000;

fn parse_args() -> Args {
    let mut args = Args::default();
    let mut argv = std::env::args().skip(1);
    while let Some(flag) = argv.next() {
        let value = argv.next().and_then(|v| v.parse::<usize>().ok());
        match (flag.as_str(), value) {
            ("--dim", Some(v)) => args.dim = v,
            ("--max-rows", Some(v)) => args.max_rows = v,
            ("--k", Some(v)) => args.k = v,
            ("--queries", Some(v)) => args.queries = v,
            _ => eprintln!("ignoring unrecognised argument {flag}"),
        }
    }
    args
}

fn main() {
    let args = parse_args();
    let device = best_device();

    println!("device      : {}", device_name(&device));
    println!("dim         : {}", args.dim);
    println!("k           : {}", args.k);
    println!(
        "gpu budget  : {:.1} GiB ({}, override with {BUDGET_ENV})",
        gpu_budget_bytes() as f64 / (1024.0 * 1024.0 * 1024.0),
        budget_source().as_str(),
    );
    if device_name(&device) == "cpu" {
        println!("\nNOTE: no GPU backend initialised — this measures the CPU");
        println!("fallback, which is correct but says nothing about VRAM.");
    }

    println!(
        "\n{:>10}  {:>10}  {:>10}  {:>10}  {:>10}  {:>8}",
        "rows", "resident", "create", "read", "update", "exact?"
    );
    println!("{}", "-".repeat(68));

    let mut rows = FIRST_ROWS;
    while rows <= args.max_rows {
        run_scale(&args, rows);
        rows *= 2;
    }

    // Reserved-versus-observed is the number that says the accounting is real:
    // if the registry has drifted from what the device actually holds, it shows
    // up here and nowhere else.
    if let Some(allocated) = device_allocated_bytes() {
        println!(
            "\nafter teardown — reserved: {:.1} MiB, metal reports allocated: {:.1} MiB",
            gpu_resident_bytes() as f64 / (1024.0 * 1024.0),
            allocated as f64 / (1024.0 * 1024.0),
        );
    }

    println!("\nResident bytes are the corpus itself (rows x dim x 4). The device");
    println!("holds one copy; a rebuild transiently holds two, which is the real");
    println!("ceiling — see the update column.");
    println!();
    println!("The budget guards *device* residency only. Building a corpus also");
    println!("costs host memory — the MemoryStore plus the GGUF encode buffer —");
    println!("and that is unguarded, so a large enough --max-rows can still be");
    println!("killed by the OS before the budget is ever consulted.");
}
