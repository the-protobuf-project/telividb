use super::Int8Tier;
use telividb_core::ScanTier;

#[test]
fn short_codes_return_an_error_rather_than_panicking() {
    // These are `pub fn` taking untrusted bytes. The slice `&codes[start..]`
    // panicked before the `Truncated` error the code meant to return could be
    // constructed, so a truncated archive aborted the process instead of
    // reporting a bad file.
    let codes = [0u8; 4];
    let result = Int8Tier::from_codes(&codes, 64, 100, &|_| true);
    assert!(
        matches!(result, Err(crate::error::Error::Truncated { .. })),
        "expected a Truncated error"
    );
}

#[test]
fn an_absent_row_past_the_end_is_not_an_error() {
    // Absent rows are skipped before any slicing, so a bitmap that marks
    // everything absent must open cleanly even with no codes at all.
    let tier = Int8Tier::from_codes(&[], 64, 10, &|_| false).expect("all absent");
    assert_eq!(tier.len(), 10);
}
