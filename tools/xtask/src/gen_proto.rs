//! Regenerating Rust from the `.proto` files, and checking it has not drifted.
//!
//! Generated code is committed rather than produced at build time, which buys a
//! toolchain-free `cargo build` and a reviewable diff — but only if the
//! committed output actually matches the protos. `gen-proto --check` is what
//! makes that true rather than hoped for; CI runs it on every change.

use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// Where `buf generate` writes, and what `--check` compares.
const GENERATED: &str = "crates/episteme-proto/src/generated";

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

    for step in [
        vec!["lint"],
        vec!["format", "--diff", "--exit-code"],
        vec!["generate"],
    ] {
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

    flatten(&root.join(GENERATED));

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

/// buf's managed mode nests output by package path; the crate expects one flat
/// directory, so collapse it.
fn flatten(dir: &PathBuf) {
    let mut moved = Vec::new();
    collect_rs(dir, &mut moved);
    for path in moved {
        if let Some(name) = path.file_name() {
            let target = dir.join(name);
            if path != target {
                let _ = std::fs::rename(&path, &target);
            }
        }
    }
    remove_empty_dirs(dir);
}

fn collect_rs(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn remove_empty_dirs(dir: &PathBuf) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            remove_empty_dirs(&path);
            let _ = std::fs::remove_dir(&path);
        }
    }
}

/// Contents of every generated file, for drift comparison.
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
