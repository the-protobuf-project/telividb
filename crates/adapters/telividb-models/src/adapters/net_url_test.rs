use super::split;
use tpp_network::UrlScheme;

#[test]
fn a_download_link_splits_into_host_path_and_query() {
    let parts = split(
        "https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/x.gguf?download=true",
    )
    .expect("a normal download link");
    assert!(matches!(parts.scheme, UrlScheme::Https));
    assert_eq!(parts.host, "huggingface.co");
    assert_eq!(
        parts.paths,
        vec!["/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/x.gguf"]
    );
    assert_eq!(
        parts.params.get("download").map(String::as_str),
        Some("true")
    );
}

#[test]
fn an_api_url_with_no_query_still_splits() {
    let parts = split("https://huggingface.co/api/models/owner/name/tree/main").expect("api url");
    assert_eq!(parts.paths, vec!["/api/models/owner/name/tree/main"]);
    assert!(parts.params.is_empty());
}

#[test]
fn anything_that_is_not_http_is_refused() {
    // A `file://` or `s3://` URL reaching the HTTP client would fail deep
    // inside it with a message about the transport rather than about the input.
    for bad in [
        "file:///etc/passwd",
        "s3://bucket/key",
        "not a url",
        "https://",
    ] {
        assert!(split(bad).is_err(), "{bad}");
    }
}
