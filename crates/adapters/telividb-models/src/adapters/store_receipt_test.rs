//! What the verification receipt is allowed to skip, and what it must not.
//!
//! Split from `store_test.rs` because it tests a different thing: that file
//! covers installing and resolving a model, this covers the cache that spares
//! the second hash. They share a store and nothing else.

use crate::adapters::ModelStore;
use crate::{Catalog, CatalogEntry};
use telividb_core::Fingerprint;

/// A catalog entry describing `body`, so the digest and size are truthful.
fn entry_for(body: &[u8]) -> CatalogEntry {
    let mut entry = Catalog::builtin()
        .recommended()
        .expect("a recommended model")
        .clone();
    entry.id = "test-model".to_owned();
    entry.digest = Fingerprint::of(body);
    entry.size_bytes = body.len() as u64;
    entry
}

/// A verified file is not hashed twice.
///
/// The point of the receipt: the second call must answer from disk metadata
/// rather than by reading 639 MB again.
#[test]
fn a_receipt_spares_the_second_hash() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = ModelStore::new(dir.path().to_path_buf());
    let entry = entry_for(b"the model's bytes");
    std::fs::write(store.path_of(&entry.id), b"the model's bytes").expect("write");

    assert!(store.is_verified(&entry), "first call hashes and passes");
    let receipt = crate::adapters::store_receipt::Receipt::path_for(&store.path_of(&entry.id));
    assert!(receipt.exists(), "the first call leaves a receipt");
    assert!(store.is_verified(&entry), "second call answers from it");
}

/// A file replaced after verification is caught.
///
/// This is the case the receipt must not paper over: same name, different
/// bytes. Size changes, so the receipt stops describing the file and the hash
/// runs again.
#[test]
fn a_replaced_file_is_re_verified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = ModelStore::new(dir.path().to_path_buf());
    let entry = entry_for(b"the model's bytes");
    let path = store.path_of(&entry.id);
    std::fs::write(&path, b"the model's bytes").expect("write");
    assert!(store.is_verified(&entry));

    std::fs::write(&path, b"something else entirely").expect("replace");
    assert!(
        !store.is_verified(&entry),
        "a replaced file must not pass on a stale receipt"
    );
}
