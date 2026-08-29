//! Enforces the documentation rule from CLAUDE.md invariant 32.
//!
//! Every crate carries `#![deny(missing_docs)]`, which catches an undocumented
//! item that is *publicly reachable*. That leaves two gaps this task closes:
//!
//! 1. **`pub` items the compiler does not see as public.** An item declared
//!    `pub` inside a private module is unreachable from outside the crate, so
//!    `missing_docs` says nothing about it — `kmeans::nearest_centroid` was
//!    `pub`, undocumented, and caught by neither the compiler nor the previous
//!    version of this check.
//!
//! 2. **Doc comments that are present but say nothing.** `/// The name.` above
//!    `fn name()` satisfies the compiler and helps no one.
//!
//! It also reports coverage as a number, which the compiler will not do.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SKIP_DIRS: [&str; 4] = ["target", ".git", "node_modules", "tests"];

/// Minimum words before a doc comment is considered to say something.
const MIN_WORDS: usize = 3;

/// Item keywords that follow `pub` and need a doc comment.
///
/// `mod` is handled separately: `pub mod foo;` is documented by the `//!` at
/// the top of `foo.rs`, which is where a module's documentation belongs.
const ITEM_KINDS: [&str; 8] = [
    "fn", "struct", "enum", "const", "trait", "type", "static", "union",
];

pub fn run() -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");
    let mut thin = Vec::new();
    let mut undocumented = Vec::new();
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
            if is_undocumented_item(&lines, i, path) {
                let shown = path.strip_prefix(&root).unwrap_or(path).to_path_buf();
                undocumented.push((shown, i + 1, line.trim().to_owned()));
            }
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

    if thin.is_empty() && undocumented.is_empty() {
        return ExitCode::SUCCESS;
    }

    if !undocumented.is_empty() {
        eprintln!(
            "\ncheck-docs: {} public item(s) with no doc comment",
            undocumented.len()
        );
        for (path, line, text) in &undocumented {
            eprintln!("  {}:{}  {text}", path.display(), line);
        }
    }
    if !thin.is_empty() {
        eprintln!(
            "\ncheck-docs: {} doc comment(s) too thin to help",
            thin.len()
        );
        for (path, line, text) in &thin {
            eprintln!("  {}:{}  {:?}", path.display(), line, text);
        }
    }
    eprintln!("\nSay what the item is for, or what breaks without it — not what");
    eprintln!("its name already says.");
    ExitCode::FAILURE
}

/// Whether line `i` declares a `pub` item with no doc comment above it.
///
/// Only bare `pub` counts. `pub(crate)` and `pub(super)` are internal wiring
/// rather than surface, and holding them to the same bar would drown the real
/// finding in noise.
fn is_undocumented_item(lines: &[&str], i: usize, path: &Path) -> bool {
    let line = lines[i].trim_start();
    let Some(rest) = line.strip_prefix("pub ") else {
        return false;
    };
    let keyword = rest
        .trim_start()
        .trim_start_matches("async ")
        .trim_start_matches("unsafe ")
        .split_whitespace()
        .next()
        .unwrap_or("");

    // `pub mod foo;` is documented from inside `foo.rs`. An inline
    // `pub mod foo { .. }` has no separate file, so it falls through to the
    // ordinary "is there a `///` above it" check below.
    if keyword == "mod" && line.trim_end().ends_with(';') {
        return !module_has_inner_docs(line, path);
    }
    if keyword != "mod" && !ITEM_KINDS.contains(&keyword) {
        return false;
    }
    // Walk back over attributes and blank lines to reach the doc comment.
    //
    // An attribute may span several lines — rustfmt breaks a long one across
    // them — so a line that is neither a doc comment nor an attribute start is
    // not necessarily the end of the walk. When one is reached, skip up to the
    // line that opened it, which is the nearest `#[` above.
    //
    // Bracket counting was tried and is wrong here: an attribute value can be a
    // string containing brackets, so the count never unwinds and every item
    // below one reads as undocumented.
    let mut j = i;
    while j > 0 {
        let previous = lines[j - 1].trim();
        if previous.starts_with("#[") || previous.starts_with("#!") || previous.is_empty() {
            j -= 1;
            continue;
        }
        if previous.starts_with("///") {
            return false;
        }
        // Possibly the tail of a multi-line attribute. Look for its opening
        // line; if there is one above, resume the walk from there.
        match attribute_start(lines, j - 1) {
            Some(open) => j = open,
            None => return true,
        }
    }
    true
}

/// The line that opened the attribute ending at `i`, if there is one.
///
/// Walks upward only through lines that are plausibly *inside* an attribute,
/// and stops at the first `#[`. Searching for the nearest `#[` at any distance
/// would find a single-line attribute two items up and jump the walk over a
/// missing doc comment — which is how this check came to pass a struct that had
/// none.
///
/// Bounded as well, because an attribute runs to a handful of lines and an
/// unbounded search would treat any unrecognised line as continuation.
fn attribute_start(lines: &[&str], i: usize) -> Option<usize> {
    const MAX_ATTRIBUTE_LINES: usize = 16;
    let floor = i.saturating_sub(MAX_ATTRIBUTE_LINES);
    for k in (floor..i).rev() {
        let line = lines[k].trim_start();
        if line.starts_with("#[") {
            return Some(k);
        }
        // A doc comment, a blank line, or another item ends the attribute's
        // reach: none of them can be its interior.
        if line.is_empty() || line.starts_with("///") || line.starts_with("//") {
            return None;
        }
    }
    None
}

/// Whether the module `pub mod foo;` names carries `//!` documentation.
///
/// A module documents itself from the inside, so an outer `///` is not the
/// convention and demanding one would fight the language rather than the gap.
fn module_has_inner_docs(declaration: &str, path: &Path) -> bool {
    let Some(name) = declaration
        .trim()
        .strip_prefix("pub mod ")
        .and_then(|rest| rest.split(';').next())
        .map(str::trim)
    else {
        return true;
    };
    let Some(dir) = path.parent() else {
        return true;
    };
    [
        dir.join(format!("{name}.rs")),
        dir.join(name).join("mod.rs"),
    ]
    .iter()
    .any(|candidate| {
        std::fs::read_to_string(candidate).is_ok_and(|text| text.trim_start().starts_with("//!"))
    })
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

#[cfg(test)]
#[path = "check_docs_test.rs"]
mod tests;
