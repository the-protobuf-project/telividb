use super::*;

#[test]
fn row_sizes_match_each_codec() {
    assert_eq!(Codec::None.row_bytes(768), 0, "no scan tier");
    assert_eq!(Codec::F16.row_bytes(768), 1536);
    assert_eq!(
        Codec::Int8.row_bytes(768),
        776,
        "codes plus scale and offset"
    );
    assert_eq!(Codec::Pq { m: 96 }.row_bytes(768), 96);
    assert_eq!(Codec::Binary.row_bytes(768), 96);
}

#[test]
fn binary_rounds_up_to_whole_bytes() {
    assert_eq!(Codec::Binary.row_bytes(100), 13);
}

#[test]
fn ratios_rank_as_expected() {
    // The sizing table the storage design turns on.
    let d = 768;
    assert!((Codec::F16.ratio(d) - 2.0).abs() < 0.01);
    assert!((Codec::Int8.ratio(d) - 3.96).abs() < 0.05);
    assert!((Codec::Pq { m: 96 }.ratio(d) - 32.0).abs() < 0.01);
    assert!((Codec::Binary.ratio(d) - 32.0).abs() < 0.01);
}

#[test]
fn only_pq_needs_a_codebook() {
    assert!(Codec::Pq { m: 8 }.needs_codebook());
    for codec in [Codec::None, Codec::F16, Codec::Int8, Codec::Binary] {
        assert!(
            !codec.needs_codebook(),
            "{codec:?} should be self-describing"
        );
    }
}

#[test]
fn no_codec_reports_a_ratio_of_one() {
    assert_eq!(Codec::None.ratio(768), 1.0);
}

#[test]
fn every_codec_round_trips_through_its_discriminant() {
    for codec in [
        Codec::None,
        Codec::F16,
        Codec::Int8,
        Codec::Pq { m: 96 },
        Codec::Binary,
    ] {
        let (tag, param) = codec.to_bytes();
        assert_eq!(Codec::from_bytes(tag, param).unwrap(), codec);
    }
}

#[test]
fn an_unknown_discriminant_is_refused() {
    assert!(Codec::from_bytes(99, 0).is_err());
}
