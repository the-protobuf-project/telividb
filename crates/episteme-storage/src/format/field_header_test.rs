use super::*;

fn model() -> Fingerprint {
    Fingerprint::of(b"bge-large-en-v1.5.gguf")
}

fn sample() -> FieldHeader {
    FieldHeader {
        dim: Dim::new(768).unwrap(),
        dtype: DType::F32,
        codec: Codec::Int8,
        metric: Metric::Cosine,
        model_fingerprint: model(),
        rows: 1_000,
    }
}

#[test]
fn round_trips() {
    assert_eq!(FieldHeader::decode(&sample().encode()).unwrap(), sample());
}

#[test]
fn round_trips_every_codec() {
    for codec in [
        Codec::None,
        Codec::F16,
        Codec::Int8,
        Codec::Pq { m: 96 },
        Codec::Binary,
    ] {
        let h = FieldHeader { codec, ..sample() };
        assert_eq!(FieldHeader::decode(&h.encode()).unwrap().codec, codec);
    }
}

#[test]
fn round_trips_every_metric() {
    for metric in [Metric::Dot, Metric::L2, Metric::Cosine] {
        let h = FieldHeader { metric, ..sample() };
        assert_eq!(FieldHeader::decode(&h.encode()).unwrap().metric, metric);
    }
}

#[test]
fn round_trips_every_dtype() {
    for dtype in [DType::F32, DType::F16, DType::BF16] {
        let h = FieldHeader { dtype, ..sample() };
        assert_eq!(FieldHeader::decode(&h.encode()).unwrap().dtype, dtype);
    }
}

#[test]
fn carries_model_provenance_intact() {
    assert_eq!(
        FieldHeader::decode(&sample().encode())
            .unwrap()
            .model_fingerprint,
        model()
    );
}

#[test]
fn row_sizes_follow_dtype_and_codec() {
    assert_eq!(sample().raw_row_bytes(), 768 * 4);
    assert_eq!(
        sample().codes_row_bytes(),
        768 + 8,
        "int8 plus scale/offset"
    );

    let half = FieldHeader {
        dtype: DType::F16,
        codec: Codec::None,
        ..sample()
    };
    assert_eq!(half.raw_row_bytes(), 768 * 2);
    assert_eq!(half.codes_row_bytes(), 0, "no scan tier");
}

#[test]
fn matching_model_is_accepted() {
    assert!(sample().check_model(model()).is_ok());
}

#[test]
fn drifted_model_is_refused() {
    // The failure that never announces itself: mixed provenance produces
    // plausible, wrong neighbours and no error at all.
    let err = sample()
        .check_model(Fingerprint::of(b"e5-large-v2.gguf"))
        .unwrap_err();
    assert!(matches!(err, Error::ModelDrift { .. }));
}

#[test]
fn unset_model_skips_the_check() {
    let unbound = FieldHeader {
        model_fingerprint: Fingerprint::unset(),
        ..sample()
    };
    assert!(unbound.check_model(model()).is_ok());
}

#[test]
fn wrong_magic_is_rejected() {
    // A field header and a segment header are the same length; without distinct
    // magic, reading one as the other would silently produce nonsense.
    let mut bytes = sample().encode();
    bytes[0..4].copy_from_slice(b"EPSG");
    assert!(matches!(
        FieldHeader::decode(&bytes),
        Err(Error::BadMagic { .. })
    ));
}

#[test]
fn newer_version_is_refused() {
    let mut bytes = sample().encode();
    bytes[4..6].copy_from_slice(&(FIELD_VERSION + 1).to_le_bytes());
    assert!(matches!(
        FieldHeader::decode(&bytes),
        Err(Error::UnsupportedVersion { .. })
    ));
}

#[test]
fn a_flipped_bit_is_caught() {
    let mut bytes = sample().encode();
    bytes[20] ^= 0b0000_0001;
    assert!(matches!(
        FieldHeader::decode(&bytes),
        Err(Error::Corrupt { .. })
    ));
}

#[test]
fn zero_dim_is_rejected_on_read() {
    let mut bytes = sample().encode();
    bytes[6..10].copy_from_slice(&0u32.to_le_bytes());
    let crc = crc32fast::hash(&bytes[..56]);
    bytes[56..60].copy_from_slice(&crc.to_le_bytes());
    assert!(FieldHeader::decode(&bytes).is_err());
}
