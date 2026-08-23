use super::*;

fn schema() -> Fingerprint {
    Fingerprint::of(b"descriptor-set-v1")
}

fn sample() -> SegmentHeader {
    SegmentHeader {
        schema_fingerprint: schema(),
        rows: 1_000_000,
        deleted: 12,
    }
}

#[test]
fn round_trips() {
    assert_eq!(SegmentHeader::decode(&sample().encode()).unwrap(), sample());
}

#[test]
fn carries_the_schema_fingerprint_intact() {
    let back = SegmentHeader::decode(&sample().encode()).unwrap();
    assert_eq!(back.schema_fingerprint, schema());
}

#[test]
fn new_starts_with_no_tombstones() {
    let h = SegmentHeader::new(schema(), 500);
    assert_eq!(h.deleted, 0);
    assert_eq!(h.live_rows(), 500);
}

#[test]
fn foreign_file_is_rejected_by_magic() {
    let mut bytes = sample().encode();
    bytes[0..4].copy_from_slice(b"PARQ");
    assert!(matches!(
        SegmentHeader::decode(&bytes),
        Err(Error::BadMagic { .. })
    ));
}

#[test]
fn newer_version_is_refused_not_guessed() {
    let mut bytes = sample().encode();
    bytes[4..6].copy_from_slice(&(SEGMENT_VERSION + 1).to_le_bytes());
    let err = SegmentHeader::decode(&bytes).unwrap_err();
    assert!(matches!(err, Error::UnsupportedVersion { found, .. } if found == SEGMENT_VERSION + 1));
}

#[test]
fn a_flipped_bit_anywhere_in_the_body_is_caught() {
    for byte in [8usize, 20, 39, 40, 50] {
        let mut bytes = sample().encode();
        bytes[byte] ^= 0b0000_0001;
        assert!(
            matches!(SegmentHeader::decode(&bytes), Err(Error::Corrupt { .. })),
            "flip at byte {byte} was not caught"
        );
    }
}

#[test]
fn truncation_is_caught() {
    let bytes = sample().encode();
    assert!(matches!(
        SegmentHeader::decode(&bytes[..40]),
        Err(Error::Truncated { .. })
    ));
}

#[test]
fn matching_schema_is_accepted() {
    assert!(sample().check_schema(schema()).is_ok());
}

#[test]
fn drifted_schema_is_refused() {
    let err = sample()
        .check_schema(Fingerprint::of(b"descriptor-set-v2"))
        .unwrap_err();
    assert!(matches!(err, Error::SchemaDrift { .. }));
}

#[test]
fn unset_fingerprint_skips_the_check_on_either_side() {
    // "Unknown", never "agrees" — but it must not block reading fixtures or
    // segments written before a schema was bound.
    let unbound = SegmentHeader::new(Fingerprint::unset(), 1);
    assert!(unbound.check_schema(schema()).is_ok());
    assert!(sample().check_schema(Fingerprint::unset()).is_ok());
}

#[test]
fn live_rows_excludes_tombstones() {
    assert_eq!(sample().live_rows(), 999_988);
}

#[test]
fn live_rows_saturates_rather_than_underflowing() {
    let h = SegmentHeader {
        rows: 5,
        deleted: 9,
        ..sample()
    };
    assert_eq!(h.live_rows(), 0);
}

#[test]
fn header_is_exactly_the_declared_size() {
    assert_eq!(sample().encode().len(), HEADER_BYTES);
}
