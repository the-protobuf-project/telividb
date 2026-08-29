use super::Reference;

#[test]
fn the_three_shapes_people_actually_paste_all_resolve() {
    // A repository page from the address bar, the `owner/name` from a README,
    // and a direct download link. Refusing two of the three because they are
    // not canonical is exactly the friction the catalog exists to remove.
    let repo = Reference::Repository {
        repo: "CompendiumLabs/bge-small-en-v1.5-gguf".to_owned(),
    };
    for input in [
        "CompendiumLabs/bge-small-en-v1.5-gguf",
        "https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf",
        "http://www.huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/",
        "  huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf  ",
        "https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/tree/main",
    ] {
        assert_eq!(Reference::parse(input), Some(repo.clone()), "{input}");
    }
}

#[test]
fn a_link_to_one_file_keeps_the_filename() {
    let parsed = Reference::parse(
        "https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/bge-small-en-v1.5-q8_0.gguf?download=true",
    );
    assert_eq!(
        parsed,
        Some(Reference::File {
            repo: "CompendiumLabs/bge-small-en-v1.5-gguf".to_owned(),
            revision: "main".to_owned(),
            file: "bge-small-en-v1.5-q8_0.gguf".to_owned(),
        })
    );
}

#[test]
fn a_blob_view_names_the_same_file_as_a_download_link() {
    // People copy from the file browser as often as from the download button.
    let Some(Reference::File { file, .. }) = Reference::parse(
        "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/blob/main/nomic-embed-text-v1.5.f16.gguf",
    ) else {
        panic!("a blob view should name a file");
    };
    assert_eq!(file, "nomic-embed-text-v1.5.f16.gguf");
}

#[test]
fn a_url_elsewhere_is_kept_as_a_url() {
    let parsed = Reference::parse("https://example.internal/models/custom.gguf");
    assert_eq!(
        parsed,
        Some(Reference::Url {
            url: "https://example.internal/models/custom.gguf".to_owned()
        })
    );
    assert_eq!(parsed.and_then(|r| r.repository().map(str::to_owned)), None);
}

#[test]
fn input_with_no_plausible_reading_is_refused() {
    for nonsense in ["", "   ", "bge-small", "a/b/c"] {
        assert_eq!(Reference::parse(nonsense), None, "{nonsense:?}");
    }
}

#[test]
fn a_link_to_a_tag_or_commit_keeps_that_revision() {
    // Normalising this to `main` would fetch a different file than the one the
    // person pasted, silently — the link names a revision precisely because the
    // contents of `main` move.
    let Some(Reference::File { revision, file, .. }) = Reference::parse(
        "https://huggingface.co/Qwen/Qwen3-Embedding-0.6B-GGUF/resolve/refs%2Fpr%2F3/Qwen3-Embedding-0.6B-Q8_0.gguf",
    ) else {
        panic!("a resolve link should name a file");
    };
    assert_eq!(revision, "refs%2Fpr%2F3");
    assert_eq!(file, "Qwen3-Embedding-0.6B-Q8_0.gguf");
}
