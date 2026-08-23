//! Repository checks that CI enforces.
//!
//! Usage: `cargo xtask check-len`

mod check_docs;
mod check_len;

fn main() -> std::process::ExitCode {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some("check-len") => check_len::run(),
        Some("check-docs") => check_docs::run(),
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
         check-docs   fail on any public item without a doc comment"
    );
}
