//! Enforces the dependency direction from CLAUDE.md invariant 14.
//!
//! *Ports point inward; adapters plug in from outside.* Two things follow, and
//! both are checked here:
//!
//! 1. **Between crates.** Dependencies point toward `core`. `core` knows about
//!    no I/O and nothing in the workspace; `server` is the composition root and
//!    may name anything. An outward dependency means a trait is in the wrong
//!    crate — the fix is to move the trait inward, never the implementation
//!    outward.
//!
//! 2. **Within a crate.** `domain` is pure types and logic, `ports` is the
//!    boundary, `adapters` are the replaceable implementations. So `domain` and
//!    `ports` must never name `adapters`: the moment domain logic knows a
//!    concrete adapter, "bring your own index" stops being true.
//!
//! Dev-dependencies and `*_test.rs` siblings are deliberately not checked. An
//! integration test in `telividb-storage` that exercises a real index, or a
//! `domain` unit test that needs a concrete store to run against, is testing
//! the seam rather than crossing it. Forbidding either would only push the test
//! somewhere less useful, which buys nothing and costs coverage.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[path = "check_layers_modules.rs"]
mod modules;
use modules::check_modules;

/// What each crate is allowed to depend on, inside the workspace.
///
/// Listed rather than derived from a rank, because the interesting constraint
/// is not "lower number" but *which* boundary a crate is allowed to see. A rank
/// would silently permit `telividb-storage` to depend on `telividb-index`, and
/// that is precisely the edge invariant 6 exists to forbid.
const ALLOWED: &[(&str, &[&str])] = &[
    ("telividb-core", &[]),
    ("telividb-proto", &[]),
    ("telividb-telemetry", &[]),
    ("telividb-graph", &["telividb-core", "telividb-telemetry"]),
    ("telividb-distance", &["telividb-core"]),
    (
        "telividb-storage",
        &["telividb-core", "telividb-distance", "telividb-telemetry"],
    ),
    (
        "telividb-index",
        &["telividb-core", "telividb-distance", "telividb-telemetry"],
    ),
    // Note what is absent: `telividb-index`. The inference server and the GPU
    // index both sit on candle, but neither may reach into the other — a
    // shared device helper would put one adapter behind the other's optional
    // feature, which is the outward dependency rule 14 forbids.
    ("telividb-embed", &["telividb-core", "telividb-telemetry"]),
    (
        "telividb-server",
        &[
            "telividb-core",
            "telividb-distance",
            "telividb-embed",
            "telividb-index",
            "telividb-proto",
            "telividb-storage",
            "telividb-telemetry",
        ],
    ),
];

/// Every crate directory two levels under `crates/` —
/// `crates/<domain|adapters|platform|bin>/<crate>/`.
fn find_crates(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(groups) = std::fs::read_dir(root.join("crates")) else {
        return found;
    };
    for group in groups.flatten() {
        let Ok(entries) = std::fs::read_dir(group.path()) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.path().join("Cargo.toml").is_file() {
                found.push(entry.path());
            }
        }
    }
    found
}

/// Check every crate and inner module, reporting each violation found.
///
/// Reports all of them rather than stopping at the first: a layering mistake
/// usually arrives as a cluster, and fixing them one build at a time is slow.
pub fn run() -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");
    let mut problems: Vec<String> = Vec::new();
    let allowed: BTreeMap<&str, &[&str]> = ALLOWED.iter().copied().collect();

    let crates = find_crates(&root);
    if crates.is_empty() {
        eprintln!("check-layers: no crates found under {}", root.display());
        return ExitCode::FAILURE;
    }

    for dir in &crates {
        let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        match allowed.get(name) {
            Some(permitted) => {
                check_manifest(&dir.join("Cargo.toml"), name, permitted, &mut problems)
            }
            None => problems.push(format!(
                "{name} is not listed in check-layers. Add it to ALLOWED with the \
                 crates it may depend on — a new crate without a declared \
                 position in the layering is how the direction gets lost."
            )),
        }
        check_modules(&dir.join("src"), name, &mut problems);
    }

    if problems.is_empty() {
        println!(
            "check-layers: {} crates, every dependency points inward",
            crates.len()
        );
        return ExitCode::SUCCESS;
    }
    eprintln!("check-layers: {} violation(s)\n", problems.len());
    for problem in &problems {
        eprintln!("  {problem}\n");
    }
    eprintln!(
        "Dependencies point inward, toward core. If you need an outward\n\
         dependency, the abstraction is in the wrong crate: move the trait\n\
         inward, not the implementation outward."
    );
    ExitCode::FAILURE
}

/// Flag any workspace dependency the crate is not permitted to name.
fn check_manifest(manifest: &Path, name: &str, permitted: &[&str], problems: &mut Vec<String>) {
    let Ok(text) = std::fs::read_to_string(manifest) else {
        return;
    };
    let mut section = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.to_owned();
            continue;
        }
        // Dev-dependencies cross the seam on purpose; see the module docs.
        if section.contains("dev-dependencies") {
            continue;
        }
        let Some(dep) = trimmed.split(['=', '.']).next().map(str::trim) else {
            continue;
        };
        if !dep.starts_with("telividb-") || dep == name {
            continue;
        }
        if !permitted.contains(&dep) {
            problems.push(format!(
                "{name} depends on {dep}, which points outward. \
                 Permitted: {permitted:?}"
            ));
        }
    }
}

#[cfg(test)]
#[path = "check_layers_test.rs"]
mod tests;
