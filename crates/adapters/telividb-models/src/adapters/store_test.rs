use super::ModelStore;
use crate::adapters::fake_fetcher::FakeFetcher;
use crate::{Catalog, CatalogEntry, Error};
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

#[test]
fn a_verified_download_lands_at_the_expected_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let store = ModelStore::new(dir.path());
    let body = b"pretend this is a gguf".to_vec();
    let entry = entry_for(&body);

    let mut seen = 0;
    let path = store
        .install(&entry, &FakeFetcher::serving(body.clone()), &mut |n| {
            seen = n;
            true
        })
        .expect("install");

    assert_eq!(path, store.path_of("test-model"));
    assert_eq!(std::fs::read(&path).expect("read back"), body);
    assert_eq!(seen, body.len() as u64);
    assert!(store.is_installed(&entry));
    assert_eq!(store.installed_ids().expect("list"), vec!["test-model"]);
}

#[test]
fn bytes_that_do_not_match_the_digest_never_reach_the_load_path() {
    // The case this exists for is not a corrupted copy of the right model: it
    // is a *different file*, after which every property the catalog records —
    // width, context, quality — is unverified. So it must not be loadable, and
    // it must not be left to be resumed into the same wrong file again.
    let dir = tempfile::tempdir().expect("temp dir");
    let store = ModelStore::new(dir.path());
    let entry = entry_for(b"what the catalog promises");

    let err = store
        .install(
            &entry,
            &FakeFetcher::serving(b"what the host actually served".to_vec()),
            &mut |_| true,
        )
        .unwrap_err();

    assert!(matches!(err, Error::DigestMismatch { .. }), "{err}");
    assert!(
        !store.path_of("test-model").exists(),
        "a bad file was left in place"
    );
    assert!(
        !dir.path().join("test-model.gguf.part").exists(),
        "a bad partial was left to be resumed"
    );
    assert!(!store.is_installed(&entry));
}

#[test]
fn installing_twice_does_not_fetch_twice() {
    let dir = tempfile::tempdir().expect("temp dir");
    let store = ModelStore::new(dir.path());
    let body = b"already here".to_vec();
    let entry = entry_for(&body);
    let fetcher = FakeFetcher::serving(body);

    store
        .install(&entry, &fetcher, &mut |_| true)
        .expect("first");
    store
        .install(&entry, &fetcher, &mut |_| true)
        .expect("second");

    assert_eq!(
        fetcher.offsets().len(),
        1,
        "the second install re-downloaded"
    );
}

#[test]
fn an_interrupted_download_resumes_rather_than_restarting() {
    // A model file is large enough that this is the difference between the
    // product working on a bad connection and not (invariant 10).
    let dir = tempfile::tempdir().expect("temp dir");
    let store = ModelStore::new(dir.path());
    let body = b"0123456789abcdef".to_vec();
    let entry = entry_for(&body);

    std::fs::create_dir_all(dir.path()).expect("mkdir");
    std::fs::write(dir.path().join("test-model.gguf.part"), &body[..6]).expect("partial");

    let fetcher = FakeFetcher::serving(body.clone());
    store
        .install(&entry, &fetcher, &mut |_| true)
        .expect("install");

    assert_eq!(
        fetcher.offsets(),
        vec![6],
        "did not resume from the partial"
    );
    assert_eq!(
        std::fs::read(store.path_of("test-model")).expect("read"),
        body
    );
}

#[test]
fn a_partial_larger_than_the_file_is_discarded_rather_than_appended_to() {
    // Left over from different weights under a reused id. Appending to it
    // produces a file that can only fail its digest, having spent the whole
    // download to get there.
    let dir = tempfile::tempdir().expect("temp dir");
    let store = ModelStore::new(dir.path());
    let body = b"the real file".to_vec();
    let entry = entry_for(&body);

    std::fs::create_dir_all(dir.path()).expect("mkdir");
    std::fs::write(
        dir.path().join("test-model.gguf.part"),
        b"a much longer leftover from something else entirely",
    )
    .expect("partial");

    let fetcher = FakeFetcher::serving(body.clone());
    store
        .install(&entry, &fetcher, &mut |_| true)
        .expect("install");

    assert_eq!(fetcher.offsets(), vec![0], "did not start over");
    assert_eq!(
        std::fs::read(store.path_of("test-model")).expect("read"),
        body
    );
}

#[test]
fn a_cancelled_install_keeps_its_partial_so_the_next_one_resumes() {
    // The point of cancelling rather than failing: the bytes already fetched
    // are good. Deleting them would make "stop" and "start over" the same
    // thing, which on a 600 MB model is the difference between a pause and an
    // hour.
    let dir = tempfile::tempdir().expect("temp dir");
    let store = ModelStore::new(dir.path());
    let body = b"0123456789abcdef".to_vec();
    let entry = entry_for(&body);

    let err = store
        .install(&entry, &FakeFetcher::serving(body.clone()), &mut |_| false)
        .unwrap_err();

    assert!(matches!(err, Error::Cancelled { .. }), "{err}");
    assert!(
        !store.path_of("test-model").exists(),
        "a partial was promoted"
    );
    let partial = dir.path().join("test-model.gguf.part");
    assert!(
        partial.exists(),
        "the partial was discarded, so nothing can resume"
    );

    // And installing again finishes it.
    store
        .install(&entry, &FakeFetcher::serving(body.clone()), &mut |_| true)
        .expect("the second attempt completes");
    assert!(store.is_installed(&entry));
}
