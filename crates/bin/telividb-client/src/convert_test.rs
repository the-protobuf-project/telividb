use super::*;

#[test]
fn a_vector_round_trips_through_the_wire_encoding() {
    let original = vec![0.1f32, -2.5, 3.75, 0.0];
    let decoded = from_wire(&to_wire(&original)).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn the_encoding_is_raw_little_endian_not_repeated_float() {
    // Four bytes per element and nothing else — no varint tags. This is the
    // property that keeps a 768-dimensional query cheap to serialize.
    let encoded = to_wire(&[1.0f32, 2.0]);
    assert_eq!(encoded.data.len(), 8);
    assert_eq!(encoded.dimensions, 2);
    assert_eq!(&encoded.data[..4], &1.0f32.to_le_bytes());
}

#[test]
fn a_declared_width_that_disagrees_with_the_bytes_is_refused() {
    // A truncation here would score the query against reinterpreted bytes and
    // return plausible, wrongly-ranked results.
    let malformed = telividb_proto::point::v1::Vector {
        data: vec![0u8; 8].into(),
        dimensions: 4,
    };
    assert!(matches!(
        from_wire(&malformed),
        Err(crate::Error::Malformed { .. })
    ));
}

#[test]
fn an_empty_vector_round_trips_rather_than_erroring() {
    let decoded = from_wire(&to_wire(&[])).unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn a_point_carries_its_vector_under_the_named_field() {
    let point = point_with_vector("text", &[1.0, 2.0]);
    assert_eq!(point.vectors.len(), 1);
    assert_eq!(point.vectors[0].field_id, "text");
    assert_eq!(point.vectors[0].vector.as_ref().unwrap().dimensions, 2);
}

#[test]
fn an_absent_or_empty_inline_text_reads_as_none() {
    // Empty is proto3's default for an unset string, so it must not be
    // reported as text the point actually carries.
    let mut point = point_with_vector("text", &[1.0]);
    assert_eq!(inline_text(&point), None);

    point.content_ref = Some(telividb_proto::point::v1::ContentRef {
        uri: "file:///x".to_owned(),
        range_start: 0,
        range_end: 0,
        sha256: Default::default(),
        inline_text: String::new(),
    });
    assert_eq!(inline_text(&point), None);
}
