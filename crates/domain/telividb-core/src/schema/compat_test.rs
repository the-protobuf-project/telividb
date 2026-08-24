use super::*;
use crate::schema::{PointType, VectorFieldSpec};
use crate::{Dim, Fingerprint, Metric};

fn field(name: &str, dim: u32, metric: Metric) -> VectorFieldSpec {
    VectorFieldSpec::new(name, Dim::new(dim).unwrap(), metric)
}

fn point(type_name: &str, fields: Vec<VectorFieldSpec>) -> PointType {
    PointType {
        type_name: type_name.to_owned(),
        pattern: "recordings/{recording}/shots/{shot}".to_owned(),
        vector_fields: fields,
        edges: vec!["HAS_SHOT".to_owned()],
        has_span: true,
    }
}

fn schema(tag: &[u8], types: Vec<PointType>) -> CollectionSchema {
    let mut s = CollectionSchema::new("media", Fingerprint::of(tag));
    s.point_types = types;
    s
}

fn base() -> CollectionSchema {
    schema(
        b"v1",
        vec![point(
            "media.telividb.dev/Shot",
            vec![field("text_bge", 768, Metric::Cosine)],
        )],
    )
}

#[test]
fn identical_fingerprints_short_circuit() {
    assert_eq!(compare(&base(), &base()), Compatibility::Identical);
}

#[test]
fn adding_a_vector_field_is_additive() {
    let after = schema(
        b"v2",
        vec![point(
            "media.telividb.dev/Shot",
            vec![
                field("text_bge", 768, Metric::Cosine),
                field("image_clip", 512, Metric::Cosine),
            ],
        )],
    );
    let verdict = compare(&base(), &after);
    assert!(verdict.is_readable());
    match verdict {
        Compatibility::Additive { added_fields, .. } => {
            assert_eq!(added_fields, vec!["media.telividb.dev/Shot.image_clip"]);
        }
        other => panic!("expected additive, got {other:?}"),
    }
}

#[test]
fn adding_a_point_type_is_additive() {
    let mut after = base();
    after.fingerprint = Fingerprint::of(b"v2");
    after.point_types.push(point(
        "media.telividb.dev/Transcript",
        vec![field("text_bge", 768, Metric::Cosine)],
    ));

    match compare(&base(), &after) {
        Compatibility::Additive {
            added_point_types, ..
        } => assert_eq!(added_point_types, vec!["media.telividb.dev/Transcript"]),
        other => panic!("expected additive, got {other:?}"),
    }
}

#[test]
fn removing_a_vector_field_is_breaking() {
    let after = schema(b"v2", vec![point("media.telividb.dev/Shot", vec![])]);
    let verdict = compare(&base(), &after);
    assert!(!verdict.is_readable());
    match verdict {
        Compatibility::Breaking { reasons } => {
            assert!(reasons[0].contains("text_bge"), "{reasons:?}");
        }
        other => panic!("expected breaking, got {other:?}"),
    }
}

#[test]
fn removing_a_point_type_is_breaking() {
    let after = schema(b"v2", vec![]);
    assert!(!compare(&base(), &after).is_readable());
}

#[test]
fn changing_dimension_is_breaking() {
    // Fixed stride and every stored row assume otherwise.
    let after = schema(
        b"v2",
        vec![point(
            "media.telividb.dev/Shot",
            vec![field("text_bge", 1024, Metric::Cosine)],
        )],
    );
    match compare(&base(), &after) {
        Compatibility::Breaking { reasons } => {
            assert!(reasons[0].contains("dimension"), "{reasons:?}");
        }
        other => panic!("expected breaking, got {other:?}"),
    }
}

#[test]
fn changing_metric_is_breaking() {
    // Cosine normalises at ingest; the stored bytes are wrong for L2.
    let after = schema(
        b"v2",
        vec![point(
            "media.telividb.dev/Shot",
            vec![field("text_bge", 768, Metric::L2)],
        )],
    );
    match compare(&base(), &after) {
        Compatibility::Breaking { reasons } => {
            assert!(reasons[0].contains("metric"), "{reasons:?}");
        }
        other => panic!("expected breaking, got {other:?}"),
    }
}

#[test]
fn changing_the_model_is_breaking() {
    // A new model is a new field, not a mutation — that is what keeps
    // immutability honest.
    let mut before = base();
    before.point_types[0].vector_fields[0].model_fingerprint = Fingerprint::of(b"bge");
    let mut after = before.clone();
    after.fingerprint = Fingerprint::of(b"v2");
    after.point_types[0].vector_fields[0].model_fingerprint = Fingerprint::of(b"e5");

    match compare(&before, &after) {
        Compatibility::Breaking { reasons } => {
            assert!(reasons[0].contains("model"), "{reasons:?}");
        }
        other => panic!("expected breaking, got {other:?}"),
    }
}

#[test]
fn unset_model_fingerprints_do_not_trigger_drift() {
    let mut after = base();
    after.fingerprint = Fingerprint::of(b"v2");
    assert!(compare(&base(), &after).is_readable());
}

#[test]
fn every_breaking_reason_is_collected_not_just_the_first() {
    let after = schema(
        b"v2",
        vec![point(
            "media.telividb.dev/Shot",
            vec![field("text_bge", 1024, Metric::L2)],
        )],
    );
    match compare(&base(), &after) {
        Compatibility::Breaking { reasons } => {
            assert_eq!(reasons.len(), 2, "dimension and metric both changed");
        }
        other => panic!("expected breaking, got {other:?}"),
    }
}

#[test]
fn additive_and_breaking_together_is_breaking() {
    let after = schema(
        b"v2",
        vec![point(
            "media.telividb.dev/Shot",
            vec![
                field("text_bge", 1024, Metric::Cosine),
                field("image_clip", 512, Metric::Cosine),
            ],
        )],
    );
    assert!(!compare(&base(), &after).is_readable());
}
