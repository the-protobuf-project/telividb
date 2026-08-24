//! Printing what the process is holding.

use telividb_telemetry::residency::{Location, ResidentKind, snapshot, total_bytes};

/// Bytes as a human-readable size.
pub fn mib(bytes: usize) -> String {
    format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
}

/// Print every resident thing — the `ollama ps` view.
///
/// Real names, not redacted ones. Rule 28 governs what reaches a *telemetry
/// pipeline*, where logs are read by people granted nothing; this is a local
/// operator asking what their own process is holding, and a table of opaque
/// tokens would answer nothing. The redaction happens at the emitter, which
/// is why the registry stores names raw.
pub fn print_residency() {
    let entries = snapshot();
    if entries.is_empty() {
        println!("nothing resident.");
        return;
    }

    println!("{:<12}  {:<8}  {:>12}  name", "kind", "where", "bytes");
    println!("{}", "-".repeat(62));
    for entry in &entries {
        println!(
            "{:<12}  {:<8}  {:>12}  {}",
            entry.kind.as_str(),
            entry.location.as_str(),
            mib(entry.bytes),
            entry.name,
        );
    }

    println!(
        "\nhost: {}   device: {}",
        mib(total_bytes(Location::Host)),
        mib(total_bytes(Location::Device)),
    );

    // Worth separating: a model and an index competing for the same device is
    // the pressure the budget exists to manage, and it is invisible in a
    // single total.
    let models = count(&entries, ResidentKind::Model);
    let indexes = count(&entries, ResidentKind::VectorIndex);
    println!("{models} model(s), {indexes} index(es) resident.");
}

/// How many entries are of one kind.
fn count(entries: &[telividb_telemetry::residency::Entry], kind: ResidentKind) -> usize {
    entries.iter().filter(|e| e.kind == kind).count()
}
