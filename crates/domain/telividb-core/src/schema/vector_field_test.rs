use super::*;
use crate::{Dim, Metric};

fn field(name: &str) -> VectorFieldSpec {
    VectorFieldSpec::new(name, Dim::new(512).unwrap(), Metric::Cosine)
}

#[test]
fn index_kind_exactness_is_declared() {
    assert!(IndexKind::Flat.is_exact());
    assert!(!IndexKind::Hnsw.is_exact());
    assert!(!IndexKind::IvfPq.is_exact());
}

#[test]
fn cross_encoder_is_detected_only_when_it_differs() {
    let mut f = field("image_clip");
    f.model = "siglip.gguf".to_owned();
    assert!(!f.needs_cross_encoder(), "no query encoder declared");

    f.query_encoder = Some("siglip.gguf".to_owned());
    assert!(
        !f.needs_cross_encoder(),
        "same model is not a cross encoder"
    );

    f.query_encoder = Some("siglip.mmproj.gguf".to_owned());
    assert!(
        f.needs_cross_encoder(),
        "the text tower is a different model"
    );
}

#[test]
fn a_new_spec_defaults_to_an_exhaustive_index() {
    // Flat is the safe default: correct everywhere, and the ground truth every
    // approximate index is measured against.
    assert_eq!(field("text_bge").index, IndexKind::Flat);
}

#[test]
fn a_new_spec_records_no_provenance_until_given_some() {
    assert!(field("text_bge").model_fingerprint.is_unset());
    assert!(field("text_bge").permission.is_none());
}
