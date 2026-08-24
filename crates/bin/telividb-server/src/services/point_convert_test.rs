use super::*;
use telividb_proto::point::v1::NamedVector;

fn name() -> ResourceName {
    ResourceName::parse("collections/media/points/doc-1").unwrap()
}

#[test]
fn a_bare_wire_point_converts_and_back() {
    let wire = WirePoint {
        name: String::new(),
        vectors: Vec::new(),
        span: None,
        content_ref: None,
    };
    let point = to_domain(name(), wire).unwrap();
    assert!(point.span.is_none());
    assert!(point.content_ref.is_none());

    let back = to_wire(point);
    assert_eq!(back.name, "collections/media/points/doc-1");
    assert!(back.span.is_none());
}

#[test]
fn named_vectors_are_carried_through() {
    // Inverted in stage 2: vectors now have a durable home, so converting them
    // is the correct behaviour rather than a silent loss.
    let wire = WirePoint {
        name: String::new(),
        vectors: vec![NamedVector {
            field_id: "text_bge".to_owned(),
            vector: Some(WireVector {
                data: vec![0u8; 8].into(),
                dimensions: 2,
            }),
        }],
        span: None,
        content_ref: None,
    };
    let point = to_domain(name(), wire).unwrap();
    assert_eq!(point.vectors.get("text_bge"), Some(&vec![0.0f32, 0.0]));
}

#[test]
fn a_vector_without_a_field_id_is_refused() {
    // Each field has its own model and metric, so a vector with no field named
    // cannot be stored anywhere meaningful.
    let wire = WirePoint {
        name: String::new(),
        vectors: vec![NamedVector {
            field_id: String::new(),
            vector: Some(WireVector {
                data: vec![0u8; 8].into(),
                dimensions: 2,
            }),
        }],
        span: None,
        content_ref: None,
    };
    assert!(to_domain(name(), wire).is_err());
}

#[test]
fn a_span_round_trips_through_the_wire_shape() {
    let wire_span = WireSpan {
        start_offset: Some(ms_to_duration(1500)),
        end_offset: Some(ms_to_duration(3000)),
    };
    let domain = span_to_domain(wire_span).unwrap();
    assert_eq!(domain.start_ms(), 1500);
    assert_eq!(domain.end_ms(), 3000);

    let back = span_to_wire(domain);
    assert_eq!(duration_to_ms(&back.start_offset.unwrap()).unwrap(), 1500);
    assert_eq!(duration_to_ms(&back.end_offset.unwrap()).unwrap(), 3000);
}

#[test]
fn a_missing_span_offset_is_rejected() {
    let wire_span = WireSpan {
        start_offset: Some(ms_to_duration(0)),
        end_offset: None,
    };
    assert!(span_to_domain(wire_span).is_err());
}

#[test]
fn a_full_content_ref_round_trips() {
    let wire = WireContentRef {
        uri: "s3://bucket/key".to_owned(),
        range_start: 10,
        range_end: 20,
        sha256: vec![7u8; 32].into(),
        inline_text: "hello".to_owned(),
    };
    let domain = content_ref_to_domain(wire.clone());
    assert_eq!(domain.uri, "s3://bucket/key");
    assert_eq!(domain.byte_range, Some((10, 20)));
    assert_eq!(domain.sha256, Some([7u8; 32]));
    assert_eq!(domain.inline.as_deref(), Some("hello"));

    let back = content_ref_to_wire(domain);
    assert_eq!(back, wire);
}

#[test]
fn a_minimal_content_ref_round_trips() {
    let wire = WireContentRef {
        uri: "file:///tmp/x".to_owned(),
        range_start: 0,
        range_end: 0,
        sha256: Vec::new().into(),
        inline_text: String::new(),
    };
    let domain = content_ref_to_domain(wire.clone());
    assert!(domain.byte_range.is_none());
    assert!(domain.sha256.is_none());
    assert!(domain.inline.is_none());

    let back = content_ref_to_wire(domain);
    assert_eq!(back, wire);
}
