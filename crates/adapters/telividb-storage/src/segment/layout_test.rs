use super::*;

#[test]
fn align_up_rounds_to_the_next_boundary() {
    assert_eq!(align_up(0, 64), 0);
    assert_eq!(align_up(1, 64), 64);
    assert_eq!(align_up(60, 64), 64);
    assert_eq!(align_up(64, 64), 64, "already aligned stays put");
    assert_eq!(align_up(65, 64), 128);
}

#[test]
fn data_begins_on_a_64_byte_boundary() {
    // The zero-copy invariant: a mapped region must cast to a float slice, so
    // row zero cannot start at an arbitrary offset.
    for header in [1usize, 36, 60, 64, 100] {
        let l = FieldLayout::new(header, 3072, 10);
        assert_eq!(
            l.data_offset % FieldLayout::ALIGN,
            0,
            "header {header} produced unaligned data at {}",
            l.data_offset
        );
        assert!(
            l.data_offset >= header as u64,
            "data must follow the header"
        );
    }
}

#[test]
fn rows_are_fixed_stride() {
    let l = FieldLayout::new(60, 3072, 4);
    assert_eq!(l.row_offset(0), 64);
    assert_eq!(l.row_offset(1), 64 + 3072);
    assert_eq!(l.row_offset(3), 64 + 3 * 3072);
}

#[test]
fn total_bytes_covers_header_padding_and_data() {
    let l = FieldLayout::new(60, 3072, 10);
    assert_eq!(l.total_bytes(), 64 + 10 * 3072);
}

#[test]
fn padding_fills_the_gap_exactly() {
    let l = FieldLayout::new(60, 3072, 1);
    assert_eq!(l.padding(60), 4, "60 -> 64");
    let aligned = FieldLayout::new(64, 3072, 1);
    assert_eq!(aligned.padding(64), 0);
}

#[test]
fn an_empty_field_still_has_an_aligned_data_offset() {
    let l = FieldLayout::new(60, 3072, 0);
    assert_eq!(l.total_bytes(), 64);
    assert_eq!(l.data_offset, 64);
}

#[test]
fn field_directories_are_namespaced_per_field() {
    let seg = std::path::Path::new("/data/media/segments/seg_00001");
    assert!(field_dir(seg, "image_clip").ends_with("vectors/image_clip"));
    assert_ne!(field_dir(seg, "a"), field_dir(seg, "b"));
}
