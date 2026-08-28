//! Repository checks that CI enforces.
//!
//! Usage: `cargo xtask check-len`

mod allowed;
mod check_docs;
mod check_layers;
mod check_len;
mod gen_buffers;
mod lint_proto;
mod proc;
mod protodoc;
mod vendor_proto;

fn main() -> std::process::ExitCode {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("check-len") => check_len::run(),
        Some("check-docs") => check_docs::run(),
        Some("check-layers") => check_layers::run(),
        Some("gen-buffers") => gen_buffers::run(false),
        Some("check-buffers") => gen_buffers::run(true),
        Some("lint-proto") => lint_proto::run(),
        Some("vendor-proto") => vendor_proto::run(),
        Some("protodoc") => protodoc::run(false),
        Some("check-protodoc") => protodoc::run(true),
        Some(other) => {
            eprintln!("unknown task: {other}");
            usage();
            std::process::ExitCode::FAILURE
        }
        None => {
            usage();
            std::process::ExitCode::FAILURE
        }
    }
}

fn usage() {
    eprintln!(
        "tasks:\n  \
         check-len    fail on any .rs file over the line limit\n  \
         check-docs    fail on an undocumented `pub` item or an empty doc comment\n  \
         check-layers  fail on an outward crate or module dependency\n  \
         gen-buffers   render capnp + flatbuffers and their Rust\n  \
         check-buffers fail if the rendered output has drifted\n  \
         lint-proto    run the AIP linter over the full import closure\n  \
         vendor-proto  vendor proto dependencies into buffers/protobuf/.deps for editors\n  \
         protodoc      write README.md for every protobuf module\n  \
         check-protodoc  fail if the committed protobuf docs are stale"
    );
}
