//! Enforces the file-length rule from CLAUDE.md invariant 13.
//!
//! The rule exists to keep one concept per file. Its failure mode is
//! fragmentation — many files each holding one function — so a file that is
//! over the limit with no meaningful seam is a signal that the abstraction is
//! wrong, not an invitation to cut at line 200.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const LIMIT: usize = 200;
const SKIP_DIRS: [&str; 3] = ["target", ".git", "node_modules"];

pub fn run() -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");
    let mut offenders = Vec::new();
    let mut checked = 0usize;

    visit(&root, &mut |path| {
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        if text.contains("@generated") {
            return;
        }
        checked += 1;
        let lines = text.lines().count();
        if lines > LIMIT {
            let shown = path.strip_prefix(&root).unwrap_or(path).to_path_buf();
            offenders.push((shown, lines));
        }
    });

    if offenders.is_empty() {
        println!("check-len: {checked} files, all within {LIMIT} lines");
        return ExitCode::SUCCESS;
    }

    offenders.sort_by_key(|(_, lines)| std::cmp::Reverse(*lines));
    eprintln!(
        "check-len: {} file(s) over {LIMIT} lines\n",
        offenders.len()
    );
    for (path, lines) in &offenders {
        eprintln!("  {:>5}  {}", lines, path.display());
    }
    eprintln!("\nSplit along a conceptual seam. If there isn't one, the");
    eprintln!("abstraction is wrong — fix the design, not the line count.");
    ExitCode::FAILURE
}

fn visit(dir: &Path, f: &mut impl FnMut(&PathBuf)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skip = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| SKIP_DIRS.contains(&n));
            if !skip {
                visit(&path, f);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            f(&path);
        }
    }
}
