use super::Catalog;
use telividb_core::Modality;

#[test]
fn the_shipped_catalog_parses() {
    // The manifest is compiled in, so this is the test that turns a malformed
    // entry into a build failure rather than a panic on someone's machine.
    // Parsing also enforces the architecture gate: an entry naming something
    // the encoder cannot load cannot reach a release.
    let catalog = Catalog::builtin();
    assert!(!catalog.entries().is_empty());
}

#[test]
fn exactly_one_model_is_recommended() {
    // Two defaults is a choice presented as a recommendation, which leaves the
    // person no better off than an unsorted list.
    let catalog = Catalog::builtin();
    let marked: Vec<_> = catalog
        .entries()
        .iter()
        .filter(|e| e.recommended)
        .map(|e| e.id.as_str())
        .collect();
    assert_eq!(marked.len(), 1, "recommended: {marked:?}");
}

#[test]
fn ids_are_unique_and_digests_are_real() {
    // An id names specific weights. Two entries sharing one is the provenance
    // mixing rule 12 forbids, and it would also collide on disk, where the id
    // is the installed filename.
    let catalog = Catalog::builtin();
    let mut ids: Vec<&str> = catalog.entries().iter().map(|e| e.id.as_str()).collect();
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count, "duplicate id in the catalog");

    for entry in catalog.entries() {
        assert!(
            !entry.digest.is_unset(),
            "{}: an unset digest verifies nothing",
            entry.id
        );
        assert!(
            entry.size_bytes > 1_000_000,
            "{}: {} bytes is too small to be a model, so the size is wrong \
             and a progress bar built on it would be too",
            entry.id,
            entry.size_bytes
        );
        assert!(
            entry.dimensions > 0 && entry.context_length > 0,
            "{}",
            entry.id
        );
    }
}

#[test]
fn every_entry_is_usable_and_text() {
    // The catalog must never offer a download that cannot be used once it
    // lands. Text is the only modality with an encoder behind it, so anything
    // else here would be exactly that.
    let catalog = Catalog::builtin();
    for entry in catalog.entries() {
        assert!(entry.is_usable(), "{} is not usable", entry.id);
        assert_eq!(entry.modality, Modality::Text, "{}", entry.id);
    }
    assert_eq!(
        catalog.by_modality(Modality::Text).count(),
        catalog.entries().len()
    );
    assert_eq!(
        catalog.by_modality(Modality::Audio).count(),
        0,
        "no audio encoder exists, so no audio entry may be offered"
    );
}

#[test]
fn urls_are_built_from_the_repository_rather_than_stored() {
    let catalog = Catalog::builtin();
    let entry = catalog.recommended().expect("a recommended model");
    assert!(entry.download_url().starts_with(&entry.repository_url()));
    assert!(entry.download_url().contains(&entry.file));
}

#[test]
fn an_unknown_id_names_itself() {
    let catalog = Catalog::builtin();
    let err = catalog.require("no-such-model").unwrap_err();
    assert!(err.to_string().contains("no-such-model"));
}

#[test]
fn an_architecture_the_encoder_cannot_read_is_refused_at_parse() {
    // The gate that keeps the catalog honest. `gemma-embedding` is a real
    // embedding model in GGUF form, which is what makes it the right thing to
    // test with: it is plausible, and it does not load here.
    let manifest = r#"
[[model]]
id = "embeddinggemma-300m"
display_name = "EmbeddingGemma"
description = "A real model this engine cannot read."
repository = "ggml-org/embeddinggemma-300m-qat-q8_0-GGUF"
file = "embeddinggemma-300M-qat-Q8_0.gguf"
digest = "0000000000000000000000000000000000000000000000000000000000000001"
size_bytes = 300000000
modality = "text"
architecture = "gemma-embedding"
dimensions = 768
context_length = 2048
quantization = "Q8_0"
license = "gemma"
recommended = false
"#;
    let err = Catalog::parse(manifest).unwrap_err().to_string();
    assert!(err.contains("gemma-embedding"), "{err}");
    assert!(
        err.contains("bert"),
        "the message must name what does work: {err}"
    );
}
