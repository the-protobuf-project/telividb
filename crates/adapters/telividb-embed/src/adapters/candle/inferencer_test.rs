use super::*;
use crate::adapters::candle::fixture::{TinyModel, write_tiny_gguf};
use telividb_core::Fingerprint;

/// A registered fixture model, plus the id it is resident under.
fn resident() -> (tempfile::TempDir, CandleInferencer, ModelId) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let mut server = CandleInferencer::new();
    let id = ModelId::new("tiny", Fingerprint::unset());
    server.register(&id, &path, Pooling::Mean).unwrap();

    // The digest the model actually loaded under, which is what a field would
    // be bound to.
    let loaded = ModelId::new("tiny", Fingerprint::of(&std::fs::read(&path).unwrap()));
    (dir, server, loaded)
}

#[test]
fn a_registered_model_is_resident_and_reports_its_width() {
    let (_dir, server, id) = resident();
    assert!(server.is_resident(&id));
    assert_eq!(server.len(), 1);
    assert_eq!(server.resident_names(), vec!["tiny"]);
    assert_eq!(server.dim(&id).unwrap().get(), TinyModel::default().hidden);
}

#[test]
fn an_unregistered_model_is_refused_rather_than_loaded_on_demand() {
    // Rule 45 forbids load-run-unload, so "not resident" is a configuration
    // problem and has to be distinguishable from a transient failure.
    let (_dir, server, _) = resident();
    let absent = ModelId::new("absent", Fingerprint::unset());

    assert!(!server.is_resident(&absent));
    match server.embed(&absent, Task::Query, &["x".to_owned()]) {
        Err(crate::Error::NotResident(name)) => assert_eq!(name, "absent"),
        other => panic!("expected NotResident, got {:?}", other.map(|v| v.len())),
    }
}

#[test]
fn embedding_returns_one_unit_vector_per_input_in_order() {
    let (_dir, server, id) = resident();
    let texts = vec!["the cat".to_owned(), "the dog sat".to_owned()];

    let vectors = server.embed(&id, Task::Document, &texts).unwrap();

    assert_eq!(vectors.len(), 2);
    for vector in &vectors {
        assert_eq!(vector.len(), TinyModel::default().hidden);
        let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "not unit length: {norm}");
    }
}

#[test]
fn normalization_happens_here_so_the_cosine_path_can_use_a_dot_product() {
    // Storage treats cosine as dot-product over pre-normalized vectors. An
    // un-normalized vector arriving there does not error; it ranks partly by
    // magnitude, which looks like a quality problem rather than a bug.
    let (_dir, server, id) = resident();
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
    let (_dir, server, id) = resident();
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
    let (_dir, server, _) = resident();
    let wrong = ModelId::new("tiny", Fingerprint::of(b"different weights"));

    assert!(!server.is_resident(&wrong));
    match server.embed(&wrong, Task::Query, &["x".to_owned()]) {
        Err(crate::Error::DigestMismatch { name, .. }) => assert_eq!(name, "tiny"),
        other => panic!("expected DigestMismatch, got {:?}", other.map(|v| v.len())),
    }
}

#[test]
fn an_unset_digest_is_accepted_for_ad_hoc_use() {
    let (_dir, server, _) = resident();
    let unpinned = ModelId::new("tiny", Fingerprint::unset());

    assert!(server.is_resident(&unpinned));
    assert!(
        server
            .embed(&unpinned, Task::Query, &["x".to_owned()])
            .is_ok()
    );
}

#[test]
fn loading_a_file_that_is_not_what_the_caller_pinned_is_refused() {
    // Checked against the bytes actually read, not asserted alongside them —
    // an identity trusted from its caller guarantees nothing.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();

    let mut server = CandleInferencer::new();
    let lying = ModelId::new("tiny", Fingerprint::of(b"not this file"));

    assert!(matches!(
        server.register(&lying, &path, Pooling::Mean),
        Err(crate::Error::DigestMismatch { .. })
    ));
}

#[test]
fn an_empty_input_list_returns_no_vectors_rather_than_erroring() {
    let (_dir, server, id) = resident();
    assert!(server.embed(&id, Task::Document, &[]).unwrap().is_empty());
}

#[test]
fn a_fresh_server_holds_nothing() {
    let server = CandleInferencer::new();
    assert!(server.is_empty());
    assert_eq!(server.len(), 0);
    assert!(server.resident_names().is_empty());
}

#[test]
fn a_model_deregisters_from_the_residency_registry_when_dropped() {
    use telividb_telemetry::residency::{ResidentKind, snapshot};

    // Scoped to a name no other test uses. The registry is process-wide and
    // the suite runs in parallel, so a global count here would race against
    // whatever else happens to be resident.
    let unique = "drop-probe-model";
    let listed = || {
        snapshot()
            .into_iter()
            .filter(|e| e.kind == ResidentKind::Model && e.name == unique)
            .count()
    };
    assert_eq!(listed(), 0, "the probe name must start unused");

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    write_tiny_gguf(&path, &TinyModel::default()).unwrap();
    {
        let mut server = CandleInferencer::new();
        server
            .register(
                &ModelId::new(unique, Fingerprint::unset()),
                &path,
                Pooling::Mean,
            )
            .unwrap();
        assert_eq!(listed(), 1, "a resident model should be listed");
    }
    assert_eq!(listed(), 0, "dropping the server should deregister it");
}
