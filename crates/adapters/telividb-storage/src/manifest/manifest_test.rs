use super::*;
use telividb_telemetry::Meter;

#[test]
fn empty_manifest_round_trips() {
    let m = Manifest::new();
    assert_eq!(Manifest::decode(&m.encode()).unwrap(), m);
}

#[test]
fn populated_manifest_round_trips() {
    let m = Manifest::new()
        .with_segment(1)
        .with_segment(7)
        .with_segment(9);
    let back = Manifest::decode(&m.encode()).unwrap();
    assert_eq!(back.segments, vec![1, 7, 9]);
    assert_eq!(back.generation, 3);
}

#[test]
fn generation_advances_on_every_change() {
    let m = Manifest::new().with_segment(1);
    assert_eq!(m.generation, 1);
    let m = m.without_segments(&[1]);
    assert_eq!(m.generation, 2);
    assert!(m.segments.is_empty());
}

#[test]
fn compaction_retires_only_its_inputs() {
    let m = Manifest::new()
        .with_segment(1)
        .with_segment(2)
        .with_segment(3)
        .without_segments(&[1, 3]);
    assert_eq!(m.segments, vec![2]);
}

#[test]
fn foreign_file_is_rejected() {
    let mut bytes = Manifest::new().encode();
    bytes[0..4].copy_from_slice(b"XXXX");
    assert!(matches!(
        Manifest::decode(&bytes),
        Err(Error::BadMagic { .. })
    ));
}

#[test]
fn newer_version_is_refused() {
    let mut bytes = Manifest::new().encode();
    bytes[4..6].copy_from_slice(&(MANIFEST_VERSION + 1).to_le_bytes());
    assert!(matches!(
        Manifest::decode(&bytes),
        Err(Error::UnsupportedVersion { .. })
    ));
}

#[test]
fn a_flipped_bit_in_the_segment_list_is_caught() {
    let mut bytes = Manifest::new().with_segment(42).encode();
    bytes[18] ^= 0b0000_0001;
    assert!(matches!(
        Manifest::decode(&bytes),
        Err(Error::Corrupt { .. })
    ));
}

#[test]
fn write_atomic_round_trips_through_the_filesystem() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("MANIFEST");
    let m = Manifest::new().with_segment(11).with_segment(12);

    m.write_atomic(&path, &Meter::disabled()).unwrap();
    assert_eq!(Manifest::read(&path).unwrap(), m);
}

#[test]
fn publishing_leaves_no_temp_file_behind() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("MANIFEST");
    Manifest::new()
        .with_segment(1)
        .write_atomic(&path, &Meter::disabled())
        .unwrap();

    let stray: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "tmp"))
        .collect();
    assert!(stray.is_empty(), "temp file was not renamed away");
}

#[test]
fn a_second_publish_replaces_the_first_wholesale() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("MANIFEST");

    let first = Manifest::new().with_segment(1);
    first.write_atomic(&path, &Meter::disabled()).unwrap();
    let second = first.clone().with_segment(2);
    second.write_atomic(&path, &Meter::disabled()).unwrap();

    let back = Manifest::read(&path).unwrap();
    assert_eq!(back, second);
    assert_eq!(back.generation, 2);
}
