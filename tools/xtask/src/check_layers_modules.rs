//! The intra-crate half of check-layers: `domain` and `ports` must never name
//! a concrete adapter. See the module docs on [`super`] for the full rule.

use std::path::{Path, PathBuf};

/// Flag `domain` or `ports` naming a concrete adapter.
pub(super) fn check_modules(src: &Path, name: &str, problems: &mut Vec<String>) {
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
