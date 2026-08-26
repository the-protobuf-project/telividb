use super::*;
use crate::domain::Task;
use crate::ports::Inferencer;
use telividb_core::Fingerprint;

/// The downloaded model registered on an inference server, if it is present.
///
/// The real file rather than a synthetic one: registration checks the digest
/// against the bytes read (rule 12), and a fixture written by this same test
/// would agree with itself no matter what the check did.
fn resident() -> Option<(GgmlInferencer, ModelId)> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/models/gguf/text/nomic-embed-text-v1.5.Q4_K_M.gguf");
    if !path.exists() {
        eprintln!("SKIPPED: run examples/models/download.sh to exercise this");
        return None;
    }

    let mut server = GgmlInferencer::new();
    let id = ModelId::new("nomic", Fingerprint::unset());
    server.register(&id, &path).unwrap();

    // The digest the model actually loaded under, which is what a field would
    // be bound to.
    let loaded = ModelId::new("nomic", Fingerprint::of(&std::fs::read(&path).unwrap()));
    Some((server, loaded))
}

#[test]
fn a_registered_model_is_resident_and_reports_its_width() {
    let Some((server, id)) = resident() else {
        return;
    };
    assert!(server.is_resident(&id));
    assert_eq!(server.len(), 1);
    assert_eq!(server.resident_names(), vec!["nomic"]);
    assert_eq!(
        server.dim(&id).unwrap().get(),
        768,
        "nomic-embed is 768-wide"
    );
}

#[test]
fn an_unregistered_model_is_refused_rather_than_loaded_on_demand() {
    // Rule 45 forbids load-run-unload, so "not resident" is a configuration
    // problem and has to be distinguishable from a transient failure.
    let Some((server, _)) = resident() else {
        return;
    };
    let absent = ModelId::new("absent", Fingerprint::unset());

    assert!(!server.is_resident(&absent));
    match server.embed(&absent, Task::Query, &["x".to_owned()]) {
        Err(crate::Error::NotResident(name)) => assert_eq!(name, "absent"),
        other => panic!("expected NotResident, got {:?}", other.map(|v| v.len())),
    }
}

#[test]
fn embedding_returns_one_unit_vector_per_input_in_order() {
    let Some((server, id)) = resident() else {
        return;
    };
    let texts = vec!["the cat".to_owned(), "the dog sat".to_owned()];

    let vectors = server.embed(&id, Task::Document, &texts).unwrap();

    assert_eq!(vectors.len(), 2);
    for vector in &vectors {
        assert_eq!(vector.len(), 768);
        let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "not unit length: {norm}");
    }
}

#[test]
fn normalization_happens_here_so_the_cosine_path_can_use_a_dot_product() {
    // Storage treats cosine as dot-product over pre-normalized vectors. An
    // un-normalized vector arriving there does not error; it ranks partly by
    // magnitude, which looks like a quality problem rather than a bug.
    let Some((server, id)) = resident() else {
        return;
    };
    let vectors = server
        .embed(&id, Task::Document, &["the cat sat".to_owned()])
        .unwrap();

    let norm: f32 = vectors[0].iter().map(|v| v * v).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4);
}

#[test]
fn the_same_text_as_document_and_as_query_embeds_differently() {
    // The task prefix is trained in. If both sides took the same prefix, this
    // would silently become an ordinary symmetric model at lower recall.
    let Some((server, id)) = resident() else {
        return;
    };
    let text = vec!["the cat".to_owned()];

    let doc = server.embed(&id, Task::Document, &text).unwrap();
    let query = server.embed(&id, Task::Query, &text).unwrap();

    assert!(
        doc[0]
            .iter()
            .zip(&query[0])
            .any(|(a, b)| (a - b).abs() > 1e-5),
        "document and query embeddings were identical"
    );
}

#[test]
fn a_pinned_digest_that_disagrees_with_the_resident_model_is_refused() {
    // Rule 12: a field bound to one model must never be served by another.
    // Mixing provenance inside one index degrades recall with nothing
    // reporting it, so the check has to fail loudly at the boundary.
    let Some((server, _)) = resident() else {
        return;
    };
    let wrong = ModelId::new("nomic", Fingerprint::of(b"different weights"));

    assert!(!server.is_resident(&wrong));
    match server.embed(&wrong, Task::Query, &["x".to_owned()]) {
        Err(crate::Error::DigestMismatch { name, .. }) => assert_eq!(name, "nomic"),
        other => panic!("expected DigestMismatch, got {:?}", other.map(|v| v.len())),
    }
}

#[test]
fn an_unset_digest_is_accepted_for_ad_hoc_use() {
    let Some((server, _)) = resident() else {
        return;
    };
    let unpinned = ModelId::new("nomic", Fingerprint::unset());

    assert!(server.is_resident(&unpinned));
    assert!(
        server
            .embed(&unpinned, Task::Query, &["x".to_owned()])
            .is_ok()
    );
}

#[test]
fn an_empty_input_list_returns_no_vectors_rather_than_erroring() {
    let Some((server, id)) = resident() else {
        return;
    };
    assert!(server.embed(&id, Task::Document, &[]).unwrap().is_empty());
}

#[test]
fn a_fresh_server_holds_nothing() {
    let server = GgmlInferencer::new();
    assert!(server.is_empty());
    assert_eq!(server.len(), 0);
    assert!(server.resident_names().is_empty());
}
