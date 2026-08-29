use super::list_gguf;
use crate::adapters::fake_fetcher::FakeFetcher;

/// A listing in the shape the host actually returns, abridged.
const LISTING: &str = r#"[
  {"type":"file","path":"README.md","size":1200},
  {"type":"file","path":"bge-small-en-v1.5-f32.gguf","size":133609568,
   "lfs":{"oid":"bf40c42ad7d89382e2e5e5b1d3a1a5f4e6c7d8a9b0c1d2e3f4a5b6c7d8e9f001","size":133609568}},
  {"type":"file","path":"bge-small-en-v1.5-q8_0.gguf","size":36806944,
   "lfs":{"oid":"ec38e8da142596baa913124ae50550de284b6916bf59577ef2f0cb9660c2f514","size":36806944}},
  {"type":"file","path":"tiny-inline.gguf","size":40}
]"#;

#[test]
fn only_verifiable_gguf_files_are_offered_smallest_first() {
    let files = list_gguf(
        "CompendiumLabs/bge-small-en-v1.5-gguf",
        &FakeFetcher::serving(LISTING),
    )
    .expect("a well-formed listing");

    // The README is not a model; `tiny-inline.gguf` has no LFS record, so the
    // host publishes no digest for it and the download could not be verified.
    // Offering it would mean downloading on trust, which is the one thing this
    // path must not do.
    let names: Vec<&str> = files.iter().map(|f| f.file.as_str()).collect();
    assert_eq!(
        names,
        vec!["bge-small-en-v1.5-q8_0.gguf", "bge-small-en-v1.5-f32.gguf"],
        "smallest first, and nothing unverifiable"
    );
    assert_eq!(files[0].size_bytes, 36_806_944);
    assert_eq!(
        files[0].digest.to_hex(),
        "ec38e8da142596baa913124ae50550de284b6916bf59577ef2f0cb9660c2f514"
    );
}

#[test]
fn the_download_url_names_the_repository_it_came_from() {
    let files = list_gguf(
        "CompendiumLabs/bge-small-en-v1.5-gguf",
        &FakeFetcher::serving(LISTING),
    )
    .expect("list");
    let url = files[0].download_url();
    assert!(
        url.contains("CompendiumLabs/bge-small-en-v1.5-gguf"),
        "{url}"
    );
    assert!(
        url.ends_with("bge-small-en-v1.5-q8_0.gguf?download=true"),
        "{url}"
    );
}

#[test]
fn a_private_or_misspelled_repository_says_so() {
    // The host answers a missing repository with an error object rather than a
    // list. Reported as what it is, so the message points at the name that was
    // typed instead of at a parse failure.
    let err = list_gguf(
        "nobody/nothing",
        &FakeFetcher::serving(r#"{"error":"Repository not found"}"#),
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("private or misspelled"), "{err}");
}

#[test]
fn an_html_page_is_not_mistaken_for_a_listing() {
    let err = list_gguf(
        "nobody/nothing",
        &FakeFetcher::serving("<html>502 Bad Gateway</html>"),
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("not JSON"), "{err}");
}
