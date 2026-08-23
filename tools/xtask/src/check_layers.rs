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
//! integration test in `episteme-storage` that exercises a real index, or a
//! `domain` unit test that needs a concrete store to run against, is testing
//! the seam rather than crossing it. Forbidding either would only push the test
//! somewhere less useful, which buys nothing and costs coverage.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// What each crate is allowed to depend on, inside the workspace.
///
/// Listed rather than derived from a rank, because the interesting constraint
/// is not "lower number" but *which* boundary a crate is allowed to see. A rank
/// would silently permit `episteme-storage` to depend on `episteme-index`, and
/// that is precisely the edge invariant 6 exists to forbid.
const ALLOWED: &[(&str, &[&str])] = &[
    ("episteme-core", &[]),
    ("episteme-proto", &[]),
    ("episteme-telemetry", &[]),
    ("episteme-distance", &["episteme-core"]),
    (
        "episteme-storage",
        &["episteme-core", "episteme-distance", "episteme-telemetry"],
    ),
    (
        "episteme-index",
        &["episteme-core", "episteme-distance", "episteme-telemetry"],
    ),
    (
        "episteme-server",
        &[
            "episteme-core",
            "episteme-distance",
            "episteme-index",
            "episteme-proto",
            "episteme-storage",
            "episteme-telemetry",
        ],
    ),
];

/// Check every crate and inner module, reporting each violation found.
///
/// Reports all of them rather than stopping at the first: a layering mistake
/// usually arrives as a cluster, and fixing them one build at a time is slow.
pub fn run() -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");
    let mut problems: Vec<String> = Vec::new();
    let allowed: BTreeMap<&str, &[&str]> = ALLOWED.iter().copied().collect();

    let crates_dir = root.join("crates");
    let Ok(entries) = std::fs::read_dir(&crates_dir) else {
        eprintln!("check-layers: no crates/ directory at {}", root.display());
        return ExitCode::FAILURE;
    };

    let mut checked = 0usize;
    for entry in entries.flatten() {
        let dir = entry.path();
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        checked += 1;

        match allowed.get(name) {
            Some(permitted) => check_manifest(&manifest, name, permitted, &mut problems),
            None => problems.push(format!(
                "{name} is not listed in check-layers. Add it to ALLOWED with the \
                 crates it may depend on — a new crate without a declared \
                 position in the layering is how the direction gets lost."
            )),
        }
        check_modules(&dir.join("src"), name, &mut problems);
    }

    if problems.is_empty() {
        println!("check-layers: {checked} crates, every dependency points inward");
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
        if !dep.starts_with("episteme-") || dep == name {
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

/// Flag `domain` or `ports` naming a concrete adapter.
fn check_modules(src: &Path, name: &str, problems: &mut Vec<String>) {
    for inner in ["domain", "ports"] {
        let dir = src.join(inner);
        if !dir.is_dir() {
            continue;
        }
        visit_rs(&dir, &mut |path| {
            // A sibling test may name an adapter: it needs something concrete
            // to exercise the domain logic against. See the module docs.
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("_test.rs"))
            {
                return;
            }
            let Ok(text) = std::fs::read_to_string(path) else {
                return;
            };
            for (number, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                if code.contains("adapters::") || code.contains("mod adapters") {
                    problems.push(format!(
                        "{name}: {}:{} — {inner} names an adapter. {inner} defines \
                         the boundary; adapters plug into it from outside.",
                        path.display(),
                        number + 1
                    ));
                }
            }
        });
    }
}

/// Call `f` for every `.rs` file under `dir`.
fn visit_rs(dir: &Path, f: &mut impl FnMut(&PathBuf)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, f);
        } else if path.extension().is_some_and(|e| e == "rs") {
            f(&path);
        }
    }
}

#[cfg(test)]
#[path = "check_layers_test.rs"]
mod tests;
