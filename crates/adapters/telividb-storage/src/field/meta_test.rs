use super::*;

fn dim(n: u32) -> Dim {
    Dim::new(n).unwrap()
}

#[test]
fn metadata_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let meta = FieldMeta {
        dim: dim(768),
        metric: Metric::Cosine,
    };
    write(dir.path(), meta).unwrap();
    assert_eq!(read(dir.path()).unwrap(), Some(meta));
}

#[test]
fn an_unwritten_field_has_no_metadata() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(read(dir.path()).unwrap(), None);
}

#[test]
fn every_metric_survives_the_byte_encoding() {
    for metric in [Metric::Dot, Metric::L2, Metric::Cosine] {
        assert_eq!(metric_of(metric_byte(metric)).unwrap(), metric);
    }
}

#[test]
fn an_unknown_metric_byte_is_refused() {
    assert!(metric_of(99).is_err());
}

#[test]
fn a_dimension_mismatch_is_refused() {
    // Opening under the wrong width would reinterpret the field's bytes.
    let meta = FieldMeta {
        dim: dim(768),
        metric: Metric::Cosine,
    };
    assert!(meta.check(dim(384), Metric::Cosine).is_err());
    assert!(meta.check(dim(768), Metric::Cosine).is_ok());
}

#[test]
fn a_metric_mismatch_is_refused() {
    // Correctly-read vectors ranked by the wrong metric are silently wrong.
    let meta = FieldMeta {
        dim: dim(768),
        metric: Metric::Cosine,
    };
    assert!(meta.check(dim(768), Metric::L2).is_err());
}

#[test]
fn a_truncated_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        FieldMeta {
            dim: dim(4),
            metric: Metric::Dot,
        },
    )
    .unwrap();
    let file = dir.path().join("field.meta");
    let bytes = std::fs::read(&file).unwrap();
    std::fs::write(&file, &bytes[..bytes.len() - 1]).unwrap();
    assert!(read(dir.path()).is_err());
}
