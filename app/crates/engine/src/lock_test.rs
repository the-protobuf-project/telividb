//! Tests for the data-directory claim.
//!
//! These run without a window and without an engine, which is the reason this
//! crate does not depend on Tauri.

use super::*;

#[test]
fn a_second_claim_on_one_directory_is_refused() {
    let dir = std::env::temp_dir().join(format!("telividb-lock-{}", std::process::id()));
    let _held = DataDirLock::acquire(&dir).expect("first claim succeeds");

    // The failure a user would otherwise meet as a storage error several layers
    // down, reported here by name instead.
    match DataDirLock::acquire(&dir) {
        Err(Error::DataDirBusy(path)) => assert_eq!(path, dir),
        Err(other) => panic!("expected DataDirBusy, got {other}"),
        Ok(_) => panic!("two processes claimed one data directory"),
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn releasing_a_claim_frees_the_directory() {
    let dir = std::env::temp_dir().join(format!("telividb-free-{}", std::process::id()));

    // A crash releases the kernel lock, so a restart must be able to claim the
    // same directory. Dropping is the closest a test gets to that.
    drop(DataDirLock::acquire(&dir).expect("first claim succeeds"));
    let again = DataDirLock::acquire(&dir);
    assert!(again.is_ok(), "a released directory stayed locked");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn claiming_creates_the_directory() {
    let dir = std::env::temp_dir().join(format!("telividb-mkdir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    // A fresh install has no data directory, and the app should not ask the
    // user to create one before it will start.
    let held = DataDirLock::acquire(&dir).expect("claim creates the directory");
    assert!(held.path().is_dir());

    drop(held);
    let _ = std::fs::remove_dir_all(&dir);
}
