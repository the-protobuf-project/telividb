//! Enforces the documentation rule from CLAUDE.md.
//!
//! Every crate carries `#![deny(missing_docs)]`, so `cargo build` already
//! catches an undocumented public item. This task exists for two things the
//! compiler will not do: report coverage as a number, and catch doc comments
//! that are present but say nothing.
//!
//! A doc comment that restates its own name — `/// The name.` above `fn name()`
//! — passes the compiler and helps no one. Those are the ones worth finding.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SKIP_DIRS: [&str; 4] = ["target", ".git", "node_modules", "tests"];

/// Minimum words before a doc comment is considered to say something.
const MIN_WORDS: usize = 3;

pub fn run() -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");
    let mut thin = Vec::new();
    let mut documented = 0usize;
    let mut files = 0usize;

    visit(&root, &mut |path| {
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        if text.contains("@generated") {
            return;
        }
        files += 1;

        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let Some(doc) = line.trim().strip_prefix("/// ") else {
                continue;
            };
            documented += 1;

            // Only judge the first line of a block; continuation lines carry
            // the detail and are often short by design.
            let previous_is_doc = i
                .checked_sub(1)
                .and_then(|j| lines.get(j))
                .is_some_and(|l| l.trim().starts_with("///"));
            if previous_is_doc {
                continue;
            }

            if doc.split_whitespace().count() < MIN_WORDS {
                let shown = path.strip_prefix(&root).unwrap_or(path).to_path_buf();
                thin.push((shown, i + 1, doc.to_owned()));
            }
        }
    });

    println!("check-docs: {documented} doc comments across {files} files");

    if thin.is_empty() {
        return ExitCode::SUCCESS;
    }

    eprintln!(
        "\ncheck-docs: {} doc comment(s) too thin to help",
        thin.len()
    );
    for (path, line, text) in &thin {
        eprintln!("  {}:{}  {:?}", path.display(), line, text);
    }
    eprintln!("\nSay what the item is for, or what breaks without it — not what");
    eprintln!("its name already says.");
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
            let is_test = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("_test.rs"));
            if !is_test {
                f(&path);
            }
        }
    }
}
