//! Generates documentation for a tree of protobuf definitions.
//!
//! For every module directory — one whose version sub-directories hold `.proto`
//! files — it writes a `README.md` describing the services, methods, messages
//! and enums found there. It then writes a top-level `README.md` summarising
//! every module and drawing the import relationships between them.
//!
//! Written against the same convention the protos follow: one package per
//! resource, versioned, with files split by role. The tool discovers that shape
//! rather than being told it, so adding a resource needs no change here.
//!
//! Everything it writes carries a "do not edit" banner and is overwritten on
//! every run. `--check` verifies the committed docs are current without writing,
//! which is what CI uses.

mod blocks;
mod model;
mod parse;
mod render;
mod summary;

use model::Module;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Directory holding the protobuf tree.
const PROTO_ROOT: &str = "buffers/protobuf";

/// Import prefix identifying a module local to this repository.
const LOCAL_PREFIX: &str = "telividb/";

/// Generate the documentation, or verify it is current.
pub fn run(check_only: bool) -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");
    let proto_root = root.join(PROTO_ROOT);

    if !proto_root.exists() {
        eprintln!("protodoc: no {PROTO_ROOT}/ directory");
        return ExitCode::FAILURE;
    }

    let modules = discover(&proto_root);
    if modules.is_empty() {
        eprintln!("protodoc: found no module directories under {PROTO_ROOT}/");
        return ExitCode::FAILURE;
    }

    let mut written = Vec::new();
    for module in &modules {
        written.push((
            proto_root.join(&module.dir).join("README.md"),
            render::module_readme(module),
        ));
    }
    written.push((
        proto_root.join("README.md"),
        summary::root_readme(&modules, LOCAL_PREFIX),
    ));

    if check_only {
        let mut stale = Vec::new();
        for (path, expected) in &written {
            let current = std::fs::read_to_string(path).unwrap_or_default();
            if current != *expected {
                stale.push(path.strip_prefix(&root).unwrap_or(path).to_path_buf());
            }
        }
        if !stale.is_empty() {
            eprintln!("protodoc: {} file(s) out of date\n", stale.len());
            for path in &stale {
                eprintln!("  {}", path.display());
            }
            eprintln!("\nRun `cargo xtask protodoc` and commit the result.");
            return ExitCode::FAILURE;
        }
        println!(
            "protodoc: {} module(s), documentation current",
            modules.len()
        );
        return ExitCode::SUCCESS;
    }

    for (path, contents) in &written {
        if let Err(e) = std::fs::write(path, contents) {
            eprintln!("protodoc: cannot write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    }
    println!(
        "protodoc: {} module(s), {} file(s) written",
        modules.len(),
        written.len()
    );
    ExitCode::SUCCESS
}

/// Find every module directory beneath `proto_root`.
///
/// A module is a directory containing version sub-directories (`v1/`, `v2/`)
/// that hold `.proto` files. Discovering the shape rather than hard-coding it
/// means a new resource needs no change here.
fn discover(proto_root: &Path) -> Vec<Module> {
    let mut modules = Vec::new();
    walk(proto_root, proto_root, &mut modules);
    modules.sort_by(|a, b| a.dir.cmp(&b.dir));
    modules
}

fn walk(dir: &Path, proto_root: &Path, out: &mut Vec<Module>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let children: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    let has_versions = children.iter().any(|p| {
        p.is_dir()
            && p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_version_dir)
    });

    if has_versions {
        let relative = dir
            .strip_prefix(proto_root)
            .unwrap_or(dir)
            .to_string_lossy()
            .replace('\\', "/");
        let mut module = Module {
            name: title_case(dir.file_name().and_then(|n| n.to_str()).unwrap_or("")),
            dir: relative,
            ..Default::default()
        };
        parse::parse_module(dir, &mut module);
        out.push(module);
        return;
    }

    for child in children.iter().filter(|p| p.is_dir()) {
        walk(child, proto_root, out);
    }
}

/// Whether a directory name looks like `v1`, `v2beta1` and so on.
fn is_version_dir(name: &str) -> bool {
    name.starts_with('v') && name.len() > 1 && name.as_bytes()[1].is_ascii_digit()
}

fn title_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
