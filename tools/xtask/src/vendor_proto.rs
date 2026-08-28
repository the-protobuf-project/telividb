//! Vendoring proto dependencies for editor import resolution.
//!
//! Separate from linting because it answers a different question: not "are
//! these protos correct" but "can a tool outside buf resolve their imports at
//! all".

use crate::proc::which;
use std::path::Path;
use std::process::{Command, ExitCode};

/// Vendor the dependency closure for editor use.
///
/// An editor extension resolves imports from a fixed list of directories; it
/// has no way to ask buf for a module's dependencies. Without them, every
/// `import "google/api/..."` fails to resolve and each failure cascades into a
/// "cannot find" for every annotation in the file — which is why one missing
/// import path can look like a hundred and sixty errors.
///
/// Writes only the dependencies. The repository's own files are removed after
/// export, because leaving copies of them beside the originals makes every
/// message appear to be declared twice.
pub fn run() -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");
    let deps = root.join("buffers/protobuf/.deps");

    if which("buf").is_none() {
        eprintln!("vendor-proto: `buf` is not installed.");
        return ExitCode::FAILURE;
    }

    let _ = std::fs::remove_dir_all(&deps);
    if let Err(e) = std::fs::create_dir_all(&deps) {
        eprintln!("vendor-proto: cannot create {}: {e}", deps.display());
        return ExitCode::FAILURE;
    }

    match Command::new("buf")
        .args(["export", ".", "-o"])
        .arg(&deps)
        .current_dir(&root)
        .status()
    {
        Ok(status) if status.success() => {}
        _ => {
            eprintln!("vendor-proto: `buf export` failed");
            return ExitCode::FAILURE;
        }
    }

    // Drop our own files; only the dependencies are wanted here.
    let _ = std::fs::remove_dir_all(deps.join("telividb"));

    let mut count = 0usize;
    count_protos(&deps, &mut count);
    println!("vendor-proto: {count} dependency file(s) in buffers/protobuf/.deps");
    println!("Configure the editor with `gapi.protoPath`; see docs/REPO_SETUP.md");
    ExitCode::SUCCESS
}

fn count_protos(dir: &Path, count: &mut usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count_protos(&path, count);
        } else if path.extension().is_some_and(|e| e == "proto") {
            *count += 1;
        }
    }
}
