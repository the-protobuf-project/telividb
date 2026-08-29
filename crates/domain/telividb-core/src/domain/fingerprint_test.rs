use super::*;

#[test]
fn is_deterministic() {
    assert_eq!(
        Fingerprint::of(b"descriptor set"),
        Fingerprint::of(b"descriptor set")
    );
}

#[test]
fn distinguishes_different_bytes() {
    assert_ne!(Fingerprint::of(b"schema v1"), Fingerprint::of(b"schema v2"));
}

#[test]
fn a_single_bit_changes_the_digest() {
    // Schema drift is often one field renamed. It must not hash the same.
    assert_ne!(Fingerprint::of(b"field_a"), Fingerprint::of(b"field_b"));
}

#[test]
fn round_trips_through_bytes() {
    let fp = Fingerprint::of(b"model.gguf");
    assert_eq!(Fingerprint::from_bytes(*fp.as_bytes()), fp);
}

#[test]
fn unset_is_recognisable_and_not_a_real_digest() {
    assert!(Fingerprint::unset().is_unset());
    assert!(!Fingerprint::of(b"anything").is_unset());
}

#[test]
fn short_form_is_readable_and_distinguishing() {
    let a = Fingerprint::of(b"alpha");
    let b = Fingerprint::of(b"beta");
    assert_eq!(a.short().len(), 12);
    assert_ne!(a.short(), b.short());
}

#[test]
fn debug_does_not_dump_all_32_bytes() {
    let rendered = format!("{:?}", Fingerprint::of(b"x"));
    assert!(rendered.starts_with("Fingerprint("));
    assert!(rendered.len() < 30, "too long for a log line: {rendered}");
}

#[test]
fn hex_round_trips_and_rejects_malformed_input() {
    let fp = Fingerprint::of(b"a gguf file");
    let hex = fp.to_hex();
    assert_eq!(hex.len(), 64);
    assert_eq!(Fingerprint::from_hex(&hex), Some(fp));

    // A truncated or mistyped digest must not parse. Accepting one would
    // produce a value that can never match the file, which presents as a
    // corrupt download rather than as the typo in a catalog entry it is.
    assert_eq!(Fingerprint::from_hex(&hex[..63]), None, "too short");
    assert_eq!(Fingerprint::from_hex(&format!("{hex}0")), None, "too long");
    assert_eq!(
        Fingerprint::from_hex(&("g".repeat(64))),
        None,
        "not hex at all"
    );
}

#[test]
fn streaming_and_whole_buffer_hashing_agree() {
    // They must, or an installed model would verify one way and fail the other
    // depending on which path happened to check it.
    let bytes: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
    assert_eq!(
        Fingerprint::of_reader(bytes.as_slice()).expect("reads"),
        Fingerprint::of(&bytes),
        "a stream longer than one buffer must hash the same as the whole slice"
    );

    // And the empty case, which is the boundary the read loop exits on.
    assert_eq!(
        Fingerprint::of_reader(&[][..]).expect("reads"),
        Fingerprint::of(&[])
    );
}
