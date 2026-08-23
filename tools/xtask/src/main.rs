//! Repository checks that CI enforces.
//!
//! Usage: `cargo xtask check-len`

mod check_docs;
mod check_len;
mod gen_proto;
mod protodoc;

fn main() -> std::process::ExitCode {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("check-len") => check_len::run(),
        Some("check-docs") => check_docs::run(),
        Some("gen-proto") => gen_proto::run(false),
        Some("check-proto") => gen_proto::run(true),
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
         check-docs    fail on any public item without a doc comment\n  \
         gen-proto     regenerate Rust from protobuf/ with buf\n  \
         check-proto   fail if the committed generated code has drifted\n  \
         protodoc      write README.md for every protobuf module\n  \
         check-protodoc  fail if the committed protobuf docs are stale"
    );
}
