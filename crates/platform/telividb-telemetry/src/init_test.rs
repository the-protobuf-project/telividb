use super::*;
use std::path::Path;

#[test]
fn recall_sampling_is_off_when_the_rate_is_zero() {
    for draw in [0.0, 0.5, 0.999] {
        assert!(!should_sample(0.0, draw));
    }
}

#[test]
fn recall_sampling_respects_the_rate() {
    assert!(should_sample(0.01, 0.005), "below the rate should sample");
    assert!(!should_sample(0.01, 0.01), "at the rate should not");
    assert!(!should_sample(0.01, 0.9), "above the rate should not");
}

#[test]
fn a_utf8_mcap_path_passes_through_unchanged() {
    let rendered = utf8_arg(Path::new("/var/log/telividb.mcap"), "--mcap").expect("valid UTF-8");
    assert_eq!(rendered, "/var/log/telividb.mcap");
}

#[cfg(unix)]
#[test]
fn a_non_utf8_mcap_path_is_refused_rather_than_mangled() {
    // `to_string_lossy` would replace the invalid byte with U+FFFD, and the
    // recorder would open a different file than the one asked for while
    // reporting the name it was given. Refusing is the only honest answer.
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let path = Path::new(OsStr::from_bytes(b"/tmp/\xff.mcap"));
    let error = utf8_arg(path, "--mcap").expect_err("must refuse a non-UTF-8 path");
    assert!(
        error.to_string().contains("not valid UTF-8"),
        "unhelpful error: {error}"
    );
}
