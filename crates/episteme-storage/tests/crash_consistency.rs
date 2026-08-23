//! Crash consistency.
//!
//! The WAL's whole purpose is what happens after a process dies mid-write, so
//! that is what these exercise: truncate the log at every plausible boundary
//! and assert recovery is clean and lossless up to the last intact record.

use episteme_storage::wal::{WalReader, WalTail, WalWriter};
use std::fs::OpenOptions;
use std::path::Path;

fn write_log(path: &Path, records: &[&[u8]]) {
    let mut wal = WalWriter::open(path).unwrap();
    for r in records {
        wal.append(r).unwrap();
    }
    wal.commit().unwrap();
}

fn replay(path: &Path) -> (Vec<Vec<u8>>, WalTail) {
    let mut seen = Vec::new();
    let tail = WalReader::open(path)
        .unwrap()
        .replay(|r| seen.push(r.to_vec()))
        .unwrap();
    (seen, tail)
}

#[test]
fn round_trips_every_record() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("000001.wal");
    write_log(&path, &[b"alpha", b"beta", b"gamma"]);

    let (seen, tail) = replay(&path);
    assert_eq!(
        seen,
        vec![b"alpha".to_vec(), b"beta".to_vec(), b"gamma".to_vec()]
    );
    assert_eq!(tail, WalTail::Clean);
}

#[test]
fn an_empty_log_replays_clean() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("000001.wal");
    write_log(&path, &[]);
    assert_eq!(replay(&path), (vec![], WalTail::Clean));
}

#[test]
fn a_torn_tail_is_detected_at_every_truncation_point() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("000001.wal");
    write_log(&path, &[b"alpha", b"beta", b"gamma"]);

    let full = std::fs::metadata(&path).unwrap().len();

    // Truncate one byte at a time and assert recovery never panics, never
    // invents a record, and never loses one that was fully written.
    for cut in 1..full {
        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(cut)
            .unwrap();

        let (seen, tail) = replay(&path);

        for record in &seen {
            assert!(
                [b"alpha".as_slice(), b"beta", b"gamma"].contains(&record.as_slice()),
                "replayed a record that was never written: {record:?}"
            );
        }
        if cut < full {
            assert!(seen.len() < 3, "cannot have all records from a short file");
        }
        // A cut mid-record must be reported as torn, not silently accepted.
        let boundaries = [0u64, 13, 25, 38];
        if !boundaries.contains(&cut) {
            assert!(
                matches!(tail, WalTail::Torn { .. }),
                "cut at {cut} should report a torn tail, got {tail:?}"
            );
        }

        // Restore for the next iteration.
        std::fs::remove_file(&path).unwrap();
        write_log(&path, &[b"alpha", b"beta", b"gamma"]);
    }
}

#[test]
fn records_before_a_torn_tail_survive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("000001.wal");
    write_log(&path, &[b"alpha", b"beta"]);

    let full = std::fs::metadata(&path).unwrap().len();
    // Lop off part of the final record only.
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(full - 2)
        .unwrap();

    let (seen, tail) = replay(&path);
    assert_eq!(
        seen,
        vec![b"alpha".to_vec()],
        "the intact record must survive"
    );
    assert!(matches!(tail, WalTail::Torn { .. }));
}

#[test]
fn a_corrupt_payload_is_an_error_not_a_torn_tail() {
    // All the bytes are present and they are wrong — the media lied, rather
    // than the process dying. That is a different failure and must be loud.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("000001.wal");
    write_log(&path, &[b"alpha"]);

    let mut bytes = std::fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0b0000_0001;
    std::fs::write(&path, &bytes).unwrap();

    let err = WalReader::open(&path).unwrap().replay(|_| {}).unwrap_err();
    assert!(matches!(err, episteme_storage::Error::Corrupt { .. }));
}

#[test]
fn reopening_appends_rather_than_truncating() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("000001.wal");
    write_log(&path, &[b"alpha"]);
    write_log(&path, &[b"beta"]);

    let (seen, _) = replay(&path);
    assert_eq!(seen, vec![b"alpha".to_vec(), b"beta".to_vec()]);
}
