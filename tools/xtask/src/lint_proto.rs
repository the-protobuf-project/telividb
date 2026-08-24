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

use crate::proc::which;
use std::path::Path;
use std::process::{Command, ExitCode};

/// Import prefix identifying files this repository owns.
const OWNED: &str = "telividb/";

/// The api-linter version CI runs, kept in step with `.github/workflows/api-lint.yml`.
///
/// Checked rather than assumed. A newer linter adds rules, so a local install
/// behind CI reports clean on protos that CI rejects — which is how AIP-191's
/// `proto-package` rule reached `main` unnoticed while a three-version-old
/// binary said everything passed.
const PINNED_VERSION: &str = "2.3.1";

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

    if let Some(installed) = linter_version()
        && installed != PINNED_VERSION
    {
        eprintln!("lint-proto: api-linter {installed} is installed, CI runs {PINNED_VERSION}.");
        eprintln!("A version behind CI reports clean on protos CI rejects. Install the pin:");
        eprintln!("  https://github.com/googleapis/api-linter/releases/tag/v{PINNED_VERSION}");
        return ExitCode::FAILURE;
    }

    let export = std::env::temp_dir().join("telividb-proto-lint");
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

/// The installed api-linter's version, or `None` if it cannot be determined.
fn linter_version() -> Option<String> {
    let output = Command::new("api-linter").arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.split_whitespace().nth(1).map(str::to_owned)
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
