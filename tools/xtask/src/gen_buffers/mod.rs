//! Rendering the protobuf schema into Cap'n Proto and FlatBuffers, and checking
//! the committed output has not drifted.
//!
//! Mirrors `gen-proto`: the generated code is committed so `cargo build` needs
//! no `capnp` and no `flatc`, and `check-buffers` is what makes "committed
//! output matches the protos" true rather than hoped for.

mod facade;
mod preamble;
mod tree;

use crate::proc::which;
use facade::lib_rs;
use preamble::PREAMBLE;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use tree::discover;

/// Where the generated Rust is committed.
const GENERATED: &str = "crates/platform/telividb-buffers/src/generated";

/// The crate's root module, written from the file listing above.
const LIB_RS: &str = "crates/platform/telividb-buffers/src/lib.rs";

/// Descriptor set the renderer reads, rebuilt from the protos each run.
const DESCRIPTORS: &str = "descriptors.binpb";

/// The same set, committed inside the crate to power gRPC reflection.
///
/// Written with the code rather than separately: reflection that described an
/// API the server no longer serves would be worse than none.
const DESCRIPTOR_BIN: &str =
    "crates/platform/telividb-buffers/src/generated/protobuf/descriptor.bin";

/// Render both targets, or verify the committed output is current.
pub fn run(check_only: bool) -> ExitCode {
    let root = std::env::current_dir().expect("cwd is readable");

    for tool in ["buf", "buffers"] {
        if which(tool).is_none() {
            eprintln!("gen-buffers: `{tool}` is not installed.");
            eprintln!("It is a development tool, not a build dependency — the");
            eprintln!("committed output compiles without it. Install it only to");
            eprintln!("change the schema:");
            eprintln!(
                "  go install github.com/the-protobuf-project/buffers/plugin/cmd/{tool}@latest"
            );
            return ExitCode::FAILURE;
        }
    }

    let before = check_only.then(|| snapshot(&root));

    // Render into an empty tree. Without this the previous run's flattened
    // files sit beside the new nested ones and both get collected, which shows
    // up as exactly twice the expected module count.
    let _ = std::fs::remove_dir_all(root.join(GENERATED));

    // The renderer reads a descriptor set rather than the proto tree, because
    // these protos import googleapis and the buffers annotations and buf has
    // already resolved both from the registry. Re-resolving here would be a
    // second opinion about what the schema is.
    // `buf generate` runs here rather than as its own task because the render
    // above clears the tree it writes into. All three renderings come from one
    // schema, so they are produced by one command — a protobuf view regenerated
    // separately is a protobuf view that can drift from the flat ones.
    if !run_tool(
        &root,
        "buf",
        &["build", "-o", DESCRIPTORS, "--as-file-descriptor-set"],
    ) || !run_tool(&root, "buf", &["generate"])
        || !run_tool(
            &root,
            "buf",
            &["build", "-o", DESCRIPTOR_BIN, "--as-file-descriptor-set"],
        )
        || !run_tool(&root, "buffers", &["generate"])
    {
        return ExitCode::FAILURE;
    }

    let modules = match discover(&root.join(GENERATED)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("gen-buffers: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = std::fs::write(root.join(LIB_RS), lib_rs(PREAMBLE, &modules)) {
        eprintln!("gen-buffers: writing lib.rs: {e}");
        return ExitCode::FAILURE;
    }

    // Format what was just written.
    //
    // Not tidiness: `cargo fmt --check` runs over the whole workspace, and
    // `capnpc-rust` does not format its output. Leaving it unformatted puts two
    // gates in permanent disagreement — the formatter rewrites the generated
    // files, the generator rewrites them back, and whichever ran last decides
    // whether CI is green. Formatting here settles it, and rustfmt is
    // deterministic so the result stays stable across runs.
    if !format_generated(&root) {
        return ExitCode::FAILURE;
    }

    match before {
        Some(prior) if prior != snapshot(&root) => {
            eprintln!("check-buffers: committed output differs from the protos.");
            eprintln!("Run `cargo xtask gen-buffers` and commit the result.");
            ExitCode::FAILURE
        }
        Some(_) => {
            println!("check-buffers: generated output is current");
            ExitCode::SUCCESS
        }
        None => {
            println!(
                "gen-buffers: {} capnp + {} flatbuffers module(s) into {GENERATED}",
                modules.capnp.len(),
                modules.flatbuffers.len()
            );
            ExitCode::SUCCESS
        }
    }
}

/// Contents of every generated file plus `lib.rs`, for drift comparison.
fn snapshot(root: &Path) -> Vec<(PathBuf, String)> {
    let mut files = vec![];
    let _ = tree::collect(&root.join(GENERATED), ".rs", &mut files);
    files.push(root.join(LIB_RS));
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            (p, text)
        })
        .collect()
}

/// Run a development tool from the repository root, reporting a clean failure.
fn run_tool(root: &Path, tool: &str, args: &[&str]) -> bool {
    match Command::new(tool).args(args).current_dir(root).status() {
        Ok(status) if status.success() => true,
        Ok(status) => {
            eprintln!("gen-buffers: `{tool} {}` failed: {status}", args.join(" "));
            false
        }
        Err(e) => {
            eprintln!("gen-buffers: running `{tool}`: {e}");
            false
        }
    }
}

/// Run rustfmt over the generated tree and the crate root it declares.
///
/// Scoped to those files rather than the workspace: this task has no business
/// reformatting hand-written code, and a developer running it should not find
/// unrelated files in their diff.
fn format_generated(root: &Path) -> bool {
    let mut files = vec![root.join(LIB_RS)];
    if tree::collect(&root.join(GENERATED), ".rs", &mut files).is_err() {
        eprintln!("gen-buffers: could not enumerate the generated tree");
        return false;
    }
    // The edition is passed explicitly because rustfmt is invoked directly
    // rather than through cargo, so it cannot read it from the manifest.
    match Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .args(&files)
        .current_dir(root)
        .status()
    {
        Ok(status) if status.success() => true,
        Ok(status) => {
            eprintln!("gen-buffers: rustfmt failed: {status}");
            false
        }
        Err(e) => {
            eprintln!("gen-buffers: running rustfmt: {e}");
            false
        }
    }
}
