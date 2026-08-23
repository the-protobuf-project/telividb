//! Two-tier search over a segment that was sealed, closed and reopened.
//!
//! Closes the loop the codecs existed outside of: until `codes.bin` is written
//! and read back, a compressed tier is demonstrable but unreachable from
//! anything persisted.

mod support;

use episteme_core::{Metric, Ordinal, ScanTier, VectorStore};
use episteme_index::{FlatIndex, OverFetch, VectorIndex, recall_at_k, two_tier_search};
use episteme_storage::format::Codec;
use episteme_storage::{SegmentReader, SegmentWriter, segment::open_tier};
use support::{DIM, corpus};

/// Seal a corpus with `codec`, reopen it, and return both tiers.
fn seal_and_reopen(
    codec: Codec,
    dir: &std::path::Path,
) -> (SegmentReader, Option<Box<dyn ScanTier>>) {
    let (store, _) = corpus();
    let schema = episteme_core::Fingerprint::of(b"schema-v1");
    let model = episteme_core::Fingerprint::of(b"model.gguf");

    let path = dir.join("seg_00001");
    let mut writer = SegmentWriter::create(&path, schema).unwrap();
    writer
        .write_field_with_codec("text_bge", &store, model, codec)
        .unwrap();
    let sealed = writer.finish().unwrap();

    let exact = SegmentReader::open_field(&sealed, "text_bge", schema).unwrap();
    let tier = open_tier(&sealed, "text_bge").unwrap();
    (exact, tier)
}

#[test]
fn a_field_without_a_codec_has_no_scan_tier() {
    // Not an error — a normal configuration. The caller searches exactly.
    let dir = tempfile::tempdir().unwrap();
    let (exact, tier) = seal_and_reopen(Codec::None, dir.path());
    assert!(tier.is_none());
    assert_eq!(exact.len(), support::ROWS);
}

#[test]
fn every_codec_round_trips_through_a_sealed_segment() {
    for codec in [
        Codec::F16,
        Codec::Int8,
        Codec::Binary,
        Codec::Pq { m: 8 },
    ] {
        let dir = tempfile::tempdir().unwrap();
        let (exact, tier) = seal_and_reopen(codec, dir.path());
        let tier = tier.unwrap_or_else(|| panic!("{codec:?} produced no tier"));

        assert_eq!(tier.len(), exact.len(), "{codec:?} row count");

        // Scores from the reopened tier must rank like the exact ones.
        let query = exact.get(Ordinal::from_row(0)).unwrap().to_vec();
        let prepared = tier.prepare(&query, Metric::Cosine).unwrap();
        let self_score = tier.score(&prepared, Ordinal::from_row(0)).unwrap();
        assert!(self_score.is_finite(), "{codec:?} produced {self_score}");
    }
}

#[test]
fn int8_two_tier_over_a_reopened_segment_matches_exhaustive_search() {
    let dir = tempfile::tempdir().unwrap();
    let (exact, tier) = seal_and_reopen(Codec::Int8, dir.path());
    let tier = tier.unwrap();
    let (_, queries) = corpus();

    let mut recalls = Vec::new();
    for q in queries.iter().take(10) {
        let truth = FlatIndex.search(&exact, q, 10, None).unwrap();
        let (hits, _) =
            two_tier_search(tier.as_ref(), &exact, q, 10, OverFetch::default(), None).unwrap();
        recalls.push(recall_at_k(&hits, &truth, 10));
    }
    let mean = recalls.iter().sum::<f64>() / recalls.len() as f64;
    assert!(mean >= 0.99, "reopened int8 two-tier recall {mean}");
}

#[test]
fn pq_carries_its_codebook_in_the_segment() {
    // A PQ code is meaningless without exactly the codebook that produced it,
    // so the codebook must survive the round trip beside the codes.
    let dir = tempfile::tempdir().unwrap();
    let (exact, tier) = seal_and_reopen(Codec::Pq { m: 8 }, dir.path());
    let tier = tier.unwrap();

    let query = exact.get(Ordinal::from_row(0)).unwrap().to_vec();
    let prepared = tier.prepare(&query, Metric::Cosine).unwrap();

    let own = tier.score(&prepared, Ordinal::from_row(0)).unwrap();
    let other = tier.score(&prepared, Ordinal::from_row(1)).unwrap();
    assert!(own > other, "a row should score best against itself");
}

#[test]
fn a_truncated_codes_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let (_, _) = seal_and_reopen(Codec::Int8, dir.path());

    let codes = dir.path().join("seg_00001/vectors/text_bge/codes.bin");
    let bytes = std::fs::read(&codes).unwrap();
    std::fs::write(&codes, &bytes[..bytes.len() / 2]).unwrap();

    assert!(open_tier(&dir.path().join("seg_00001"), "text_bge").is_err());
}

#[test]
fn the_scan_tier_is_smaller_than_full_precision() {
    let dir = tempfile::tempdir().unwrap();
    let (_, _) = seal_and_reopen(Codec::Int8, dir.path());

    let field = dir.path().join("seg_00001/vectors/text_bge");
    let raw = std::fs::metadata(field.join("raw.bin")).unwrap().len();
    let codes = std::fs::metadata(field.join("codes.bin")).unwrap().len();

    assert!(codes < raw / 3, "codes {codes} vs raw {raw} at dim {DIM}");
}
