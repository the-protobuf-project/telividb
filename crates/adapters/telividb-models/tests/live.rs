//! Checks that reach the real model host.
//!
//! `#[ignore]` throughout, and deliberately: CI must not depend on a third
//! party being up, and these move real megabytes. They exist because the parts
//! a fake fetcher cannot prove — that the host honours range requests, that the
//! curated digests match what it serves today, that a chunked download
//! reassembles byte-for-byte — are exactly the parts most likely to be wrong.
//!
//! Run them deliberately:
//!
//! ```text
//! cargo test -p telividb-models --features network -- --ignored --nocapture
//! ```
#![cfg(feature = "network")]

use telividb_models::{Catalog, Fetcher, GgufHeader, HttpFetcher, ModelStore, huggingface};

#[test]
#[ignore = "reaches the network"]
fn every_catalog_entry_still_matches_what_the_host_serves() {
    // A catalog entry is a claim about a file someone else hosts. Weights get
    // re-quantized and re-uploaded under the same name, and when that happens
    // the digest here stops matching — which this reports as a list rather
    // than as one failed install months later.
    let fetcher = HttpFetcher::new().expect("a fetcher");
    let catalog = Catalog::builtin();

    for entry in catalog.entries() {
        let files = huggingface::list_gguf(&entry.repository, &fetcher)
            .unwrap_or_else(|e| panic!("{}: {e}", entry.id));
        let found = files
            .iter()
            .find(|f| f.file == entry.file)
            .unwrap_or_else(|| {
                panic!(
                    "{}: {} is gone from {}",
                    entry.id, entry.file, entry.repository
                )
            });

        assert_eq!(found.digest, entry.digest, "{}: digest drifted", entry.id);
        assert_eq!(
            found.size_bytes, entry.size_bytes,
            "{}: size drifted",
            entry.id
        );
        println!(
            "  ok  {:24} {:>7.1} MB",
            entry.id,
            entry.size_bytes as f64 / 1e6
        );
    }
}

#[test]
#[ignore = "reaches the network"]
fn a_header_can_be_judged_without_downloading_the_model() {
    // The whole reason the gate is cheap: a range request reads the
    // architecture out of the first couple of megabytes, so an unusable model
    // is refused before its bytes move rather than after.
    let fetcher = HttpFetcher::new().expect("a fetcher");
    let catalog = Catalog::builtin();

    for entry in catalog.entries() {
        let prefix = fetcher
            .range(&entry.download_url(), 0, GgufHeader::PREFIX_BYTES)
            .unwrap_or_else(|e| panic!("{}: {e}", entry.id));
        assert!(
            (prefix.len() as u64) < entry.size_bytes || entry.size_bytes < GgufHeader::PREFIX_BYTES,
            "{}: the host ignored the range and sent everything",
            entry.id
        );

        let header = GgufHeader::parse(&prefix).unwrap_or_else(|e| panic!("{}: {e}", entry.id));
        assert_eq!(
            header.architecture,
            entry.architecture.as_str(),
            "{}",
            entry.id
        );
        assert_eq!(
            header.dimensions,
            Some(entry.dimensions),
            "{}: the catalog's width disagrees with the file",
            entry.id
        );
        println!(
            "  ok  {:24} arch={:11} dim={:?} from {} KB",
            entry.id,
            header.architecture,
            header.dimensions,
            prefix.len() / 1024
        );
    }
}

#[test]
#[ignore = "downloads a real model"]
fn the_smallest_model_installs_and_verifies() {
    // End to end, for real: chunked ranged GETs, reassembled on disk, digest
    // checked over the bytes that arrived. The smallest entry, because the
    // point is to prove the path rather than to move the most data.
    let fetcher = HttpFetcher::new().expect("a fetcher");
    let catalog = Catalog::builtin();
    let entry = catalog
        .entries()
        .iter()
        .min_by_key(|e| e.size_bytes)
        .expect("a catalog entry");

    let dir = tempfile::tempdir().expect("temp dir");
    let store = ModelStore::new(dir.path());

    let mut last = 0;
    let path = store
        .install(entry, &fetcher, &mut |written| {
            if written - last > 8_000_000 {
                println!("    {:>6.1} MB", written as f64 / 1e6);
                last = written;
            }
        })
        .expect("install");

    let bytes = std::fs::metadata(&path).expect("stat").len();
    assert_eq!(bytes, entry.size_bytes, "the file is not the expected size");
    assert!(store.is_installed(entry), "the digest did not verify");

    // And what landed is loadable, which is the point of the whole exercise.
    let head = std::fs::read(&path).expect("read");
    let header = GgufHeader::parse(&head).expect("the installed file is a GGUF");
    header
        .require_supported(&entry.id)
        .expect("the installed file is loadable");
    println!(
        "  installed {} ({} bytes) and it is a {}",
        entry.id, bytes, header.architecture
    );
}
