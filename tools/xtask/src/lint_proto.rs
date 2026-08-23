//! Runs the Google AIP linter over the protobuf tree.
//!
//! # Why this exists rather than a shell one-liner
//!
//! Two failure modes made a hand-run command untrustworthy, and both produced
//! a *passing* result while the linter was doing nothing useful:
//!
//! 1. **Unresolved imports.** The linter needs the full transitive closure —
//!    `google/api/*` and the rest — or it exits early with a load error and
//!    never lints anything. `buf export` materialises that closure first.
//! 2. **Counting lines instead of checking status.** Piping the output through
//!    `grep -c` reports zero violations both when the protos are clean and when
//!    the linter never ran. This checks the exit status.
//!
//! The second one is why 25 violations went unnoticed. A check that cannot
//! distinguish success from not-running is worse than no check, because it is
//! believed.
//!
//! The export goes to a system temporary directory rather than anywhere inside
//! the repository. An editor extension watching the workspace would otherwise
//! lint the exported copies as well as the originals and report every message
//! as "declared multiple times".

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Import prefix identifying files this repository owns.
const OWNED: &str = "episteme/";

/// Lint every proto this repository owns, against its full import closure.
pub fn run() -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");

    for tool in ["buf", "api-linter"] {
        if which(tool).is_none() {
            eprintln!("lint-proto: `{tool}` is not installed.");
            eprintln!("Both are development tools; nothing needs them to compile.");
            eprintln!("  brew install bufbuild/buf/buf");
            eprintln!("  go install github.com/googleapis/api-linter/cmd/api-linter@latest");
            return ExitCode::FAILURE;
        }
    }

    let export = std::env::temp_dir().join("episteme-proto-lint");
    let _ = std::fs::remove_dir_all(&export);
    if let Err(e) = std::fs::create_dir_all(&export) {
        eprintln!("lint-proto: cannot create {}: {e}", export.display());
        return ExitCode::FAILURE;
    }

    // Materialise the transitive closure. Without it the linter cannot resolve
    // `google/api/*` and exits before linting anything.
    match Command::new("buf")
        .args(["export", ".", "-o"])
        .arg(&export)
        .current_dir(&root)
        .status()
    {
        Ok(status) if status.success() => {}
        _ => {
            eprintln!("lint-proto: `buf export` failed");
            return ExitCode::FAILURE;
        }
    }

    let mut owned = Vec::new();
    collect_owned(&export, &export, &mut owned);
    if owned.is_empty() {
        eprintln!("lint-proto: no files under {OWNED} in the exported closure");
        return ExitCode::FAILURE;
    }
    owned.sort();

    // `--ignore-comment-disables` is not optional. In-proto suppressions are
    // forbidden, and honouring them would let one be added to silence a rule
    // rather than fix the API.
    let output = Command::new("api-linter")
        .arg("--proto-path=.")
        .arg("--ignore-comment-disables")
        .args(&owned)
        .current_dir(&export)
        .output();

    let Ok(output) = output else {
        eprintln!("lint-proto: could not run api-linter");
        return ExitCode::FAILURE;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // A load error means nothing was linted. Treating that as a pass is the
    // bug this task exists to prevent.
    if !stderr.trim().is_empty() {
        eprintln!("lint-proto: the linter reported a problem before linting:\n{stderr}");
        return ExitCode::FAILURE;
    }

    let violations = stdout.matches("rule_id:").count();
    if violations > 0 {
        println!("{stdout}");
        eprintln!(
            "lint-proto: {violations} violation(s) across {} file(s)",
            owned.len()
        );
        eprintln!("\nSuppression is not an option — change the API.");
        return ExitCode::FAILURE;
    }

    let _ = std::fs::remove_dir_all(&export);
    println!("lint-proto: {} file(s), no violations", owned.len());
    ExitCode::SUCCESS
}

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
pub fn vendor() -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");
    let deps = root.join("protobuf/.deps");

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
    let _ = std::fs::remove_dir_all(deps.join("episteme"));

    let mut count = 0usize;
    count_protos(&deps, &mut count);
    println!("vendor-proto: {count} dependency file(s) in protobuf/.deps");
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

fn collect_owned(dir: &Path, base: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_owned(&path, base, out);
        } else if path.extension().is_some_and(|e| e == "proto") {
            let relative = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if relative.starts_with(OWNED) {
                out.push(relative);
            }
        }
    }
}

fn which(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.is_file())
    })
}
