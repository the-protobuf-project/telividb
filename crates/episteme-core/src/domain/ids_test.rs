use super::*;

#[test]
fn zero_dim_is_rejected() {
    assert!(matches!(Dim::new(0), Err(crate::Error::ZeroDim)));
}

#[test]
fn dim_round_trips() {
    assert_eq!(Dim::new(768).unwrap().get(), 768);
}

#[test]
fn ordinal_round_trips() {
    assert_eq!(Ordinal::from_row(42).row(), 42);
}
