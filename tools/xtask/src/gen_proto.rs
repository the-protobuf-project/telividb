//! Regenerating Rust from the `.proto` files, and checking it has not drifted.
//!
//! Generated code is committed rather than produced at build time, which buys a
//! toolchain-free `cargo build` and a reviewable diff — but only if the
//! committed output actually matches the protos. `gen-proto --check` is what
//! makes that true rather than hoped for; CI runs it on every change.

use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// Where `buf generate` writes, and what `--check` compares.
const GENERATED: &str = "crates/platform/telividb-proto/src/generated";

/// Run `buf` to regenerate, or verify the committed output is current.
pub fn run(check_only: bool) -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");

    if which("buf").is_none() {
        eprintln!("gen-proto: `buf` is not installed.");
        eprintln!("It is a development tool, not a build dependency — nothing");
        eprintln!("here needs it to compile. Install it only to change protos:");
        eprintln!("  brew install bufbuild/buf/buf");
        return ExitCode::FAILURE;
    }

    let before = check_only.then(|| snapshot(&root.join(GENERATED)));

    // No `buf lint`. The AIP linter is the authority on the API surface, and a
    // second linter would eventually be reconciled with a suppression.
    for step in [vec!["format", "--diff", "--exit-code"], vec!["generate"]] {
        if !run_buf(&root, &step) {
            return ExitCode::FAILURE;
        }
    }

    // The descriptor set powers gRPC reflection, so it is regenerated with the
    // code rather than separately — otherwise reflection would describe an API
    // the server no longer serves.
    if !run_buf(
        &root,
        &["build", "-o", &format!("{GENERATED}/descriptor.bin")],
    ) {
        return ExitCode::FAILURE;
    }

    if let Some(before) = before {
        let after = snapshot(&root.join(GENERATED));
        if before != after {
            eprintln!("\ngen-proto: committed output does not match the protos.");
            eprintln!("Run `cargo xtask gen-proto` and commit the result.");
            return ExitCode::FAILURE;
        }
        println!("gen-proto: committed output is current");
    } else {
        println!("gen-proto: regenerated into {GENERATED}");
    }
    ExitCode::SUCCESS
}

fn run_buf(root: &PathBuf, args: &[&str]) -> bool {
    match Command::new("buf").args(args).current_dir(root).status() {
        Ok(status) if status.success() => true,
        Ok(_) => {
            eprintln!("gen-proto: `buf {}` failed", args.join(" "));
            false
        }
        Err(e) => {
            eprintln!("gen-proto: could not run buf: {e}");
            false
        }
    }
}

/// Contents of every generated file, for drift comparison.
///
/// Walks the tree, because generation now writes one directory per package
/// rather than a single flat output.
fn snapshot(dir: &PathBuf) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Ok(bytes) = std::fs::read(&path)
        {
            out.push((
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                bytes,
            ));
        }
    }
    out.sort();
    out
}

fn which(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.is_file())
    })
}
