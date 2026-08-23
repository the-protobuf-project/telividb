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
