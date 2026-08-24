//! A published segment is complete, or it does not exist.
//!
//! The sealed-segment design rests on one property: the rename that publishes a
//! segment happens only after everything inside it is durable. If a sidecar
//! file is written without `fsync`, a crash can publish a segment whose
//! `raw.bin` is intact and whose presence bitmap or codebook is missing or
//! zero-length — and nothing downstream is prepared for that, because the whole
//! point of immutability is that a published segment never changes again.
//!
//! These tests assert the property from the outside: after sealing, every file
//! a reader will look for exists and is non-empty, and the temp directory is
//! gone.

use episteme_core::{Dim, Metric};
use episteme_index::adapters::MemoryStore;
use episteme_storage::format::Codec;
use episteme_storage::{SegmentWriter, field_dir, open_tier};
use std::path::Path;

const DIM: u32 = 8;

/// SplitMix64, so the fixture reproduces exactly and needs no dependency.
fn next_f32(state: &mut u64) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    ((z >> 40) as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
}

/// A store with enough rows to train a PQ codebook.
fn store(rows: usize, absent_every: usize) -> MemoryStore {
    let mut rng = 9u64;
    let mut store = MemoryStore::new(Dim::new(DIM).unwrap(), Metric::Dot);
    for row in 0..rows {
        if absent_every != 0 && row % absent_every == 0 {
            store.push_absent();
        } else {
            let v: Vec<f32> = (0..DIM).map(|_| next_f32(&mut rng)).collect();
            store.push(&v).unwrap();
        }
    }
    store
}

/// Seal one field into a fresh segment and return its directory.
fn seal(
    dir: &Path,
    name: &str,
    codec: Codec,
    rows: usize,
    absent_every: usize,
) -> std::path::PathBuf {
    let path = dir.join("segment.1");
    let mut writer =
        SegmentWriter::create(&path, episteme_core::Fingerprint::unset()).expect("create");
    let s = store(rows, absent_every);
    writer
        .write_field_with_codec(name, &s, episteme_core::Fingerprint::unset(), codec)
        .expect("write field");
    writer.finish().expect("finish")
}

#[test]
fn every_sidecar_a_reader_needs_is_present_and_non_empty() {
    // `present.roar` and `codebook.pq` were written with `fs::write`, which
    // does not sync. A crash between the write and the rename could publish a
    // segment missing either one.
    let dir = tempfile::tempdir().unwrap();
    let segment = seal(dir.path(), "text", Codec::Pq { m: 4 }, 400, 7);

    let field = field_dir(&segment, "text");
    for sidecar in ["raw.bin", "codes.bin", "present.roar", "codebook.pq"] {
        let path = field.join(sidecar);
        let meta = std::fs::metadata(&path)
            .unwrap_or_else(|e| panic!("{sidecar} missing from a published segment: {e}"));
        assert!(meta.len() > 0, "{sidecar} was published zero-length");
    }
    assert!(segment.join("header.bin").is_file());
}

#[test]
fn the_temp_directory_does_not_survive_a_seal() {
    // A leftover `.building` directory means the rename did not happen, or
    // happened from somewhere else.
    let dir = tempfile::tempdir().unwrap();
    let segment = seal(dir.path(), "text", Codec::Int8, 400, 0);

    let mut building = segment.as_os_str().to_os_string();
    building.push(".building");
    assert!(
        !Path::new(&building).exists(),
        "the temp directory outlived the seal"
    );
}

#[test]
fn a_published_segment_reads_back_through_the_tier() {
    // The end-to-end property: everything the scan tier needs was written,
    // synced and named before the segment became visible.
    let dir = tempfile::tempdir().unwrap();
    let segment = seal(dir.path(), "text", Codec::Pq { m: 4 }, 400, 5);

    let tier = open_tier(&segment, "text")
        .expect("open the tier")
        .expect("a codec was written, so there is a tier");
    assert_eq!(tier.len(), 400);
}

#[test]
fn two_segments_in_one_directory_do_not_share_a_temp_path() {
    // `Path::with_extension` *replaces* an extension, so `segment.1` and
    // `segment.2` both built in `segment.building` — and the second seal
    // deleted the first's work out from under it.
    let dir = tempfile::tempdir().unwrap();
    let s = store(64, 0);

    let first = dir.path().join("segment.1");
    let second = dir.path().join("segment.2");
    let mut a = SegmentWriter::create(&first, episteme_core::Fingerprint::unset()).unwrap();
    let mut b = SegmentWriter::create(&second, episteme_core::Fingerprint::unset()).unwrap();

    a.write_field("text", &s, episteme_core::Fingerprint::unset())
        .unwrap();
    b.write_field("text", &s, episteme_core::Fingerprint::unset())
        .unwrap();

    let a_path = a.finish().expect("first seal");
    let b_path = b.finish().expect("second seal");
    assert_ne!(a_path, b_path);
    assert!(a_path.join("header.bin").is_file(), "first segment lost");
    assert!(b_path.join("header.bin").is_file(), "second segment lost");
}

#[test]
fn no_codec_writes_no_codes_file() {
    // A stray zero-length `codes.bin` inside a sealed, immutable segment.
    let dir = tempfile::tempdir().unwrap();
    let segment = seal(dir.path(), "text", Codec::None, 64, 0);
    let field = field_dir(&segment, "text");
    assert!(
        !field.join("codes.bin").exists(),
        "Codec::None left a codes.bin behind"
    );
    assert!(open_tier(&segment, "text").unwrap().is_none());
}
