use super::Modality;

#[test]
fn only_text_reports_itself_supported() {
    // The catalog and the UI both route on this. If it ever answered `true`
    // for a modality with no encoder behind it, the window would offer a
    // download that cannot be used once it lands.
    assert!(Modality::Text.is_supported());
    for unreachable in [Modality::Image, Modality::Audio, Modality::Video] {
        assert!(!unreachable.is_supported(), "{unreachable}");
    }
}

#[test]
fn names_round_trip() {
    for m in [
        Modality::Text,
        Modality::Image,
        Modality::Audio,
        Modality::Video,
    ] {
        assert_eq!(Modality::parse(m.as_str()), Some(m));
    }
    assert_eq!(Modality::parse("hologram"), None);
}
