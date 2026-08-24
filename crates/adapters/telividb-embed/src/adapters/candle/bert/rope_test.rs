use super::*;

#[test]
fn rotation_preserves_the_length_of_every_head() {
    // RoPE is a rotation: it must not change magnitude. A scale factor
    // creeping in would leave every vector plausible and slowly break the
    // normalization the cosine path depends on.
    let device = Device::Cpu;
    let rope = Rope::new(8, 16, 1000.0, &device).unwrap();
    let x = Tensor::arange(0f32, 32.0, &device)
        .unwrap()
        .reshape((1, 1, 4, 8))
        .unwrap();

    let out = rope.apply(&x).unwrap();
    let before = x.sqr().unwrap().sum(D::Minus1).unwrap();
    let after = out.sqr().unwrap().sum(D::Minus1).unwrap();

    let a = before.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    let b = after.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    for (x, y) in a.iter().zip(&b) {
        assert!((x - y).abs() < 1e-3, "magnitude changed: {a:?} vs {b:?}");
    }
}

#[test]
fn position_zero_is_left_untouched() {
    // cos(0) = 1, sin(0) = 0, so the first position is the identity. If it
    // isn't, the frequency table is built wrong.
    let device = Device::Cpu;
    let rope = Rope::new(4, 8, 1000.0, &device).unwrap();
    let x = Tensor::from_vec(vec![1.0f32, 2.0, 3.0, 4.0], (1, 1, 1, 4), &device).unwrap();

    let out = rope.apply(&x).unwrap().flatten_all().unwrap();
    let values = out.to_vec1::<f32>().unwrap();
    for (got, want) in values.iter().zip([1.0, 2.0, 3.0, 4.0]) {
        assert!((got - want).abs() < 1e-5, "got {values:?}");
    }
}

#[test]
fn later_positions_are_rotated_differently() {
    // The whole point: identical content at different positions must produce
    // different keys, or the model is position-blind.
    let device = Device::Cpu;
    let rope = Rope::new(4, 8, 1000.0, &device).unwrap();
    let repeated = Tensor::from_vec(
        vec![1.0f32, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0],
        (1, 1, 2, 4),
        &device,
    )
    .unwrap();

    let out = rope.apply(&repeated).unwrap();
    let rows = out.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    assert!(
        rows[..4]
            .iter()
            .zip(&rows[4..])
            .any(|(a, b)| (a - b).abs() > 1e-4),
        "the same content at two positions rotated identically: {rows:?}"
    );
}

#[test]
fn the_non_interleaved_convention_splits_the_head_in_half() {
    // nomic-bert is trained non-interleaved (GPT-NeoX): halves rotate against
    // each other. The interleaved (GPT-J) convention pairs *adjacent*
    // elements, and picking the wrong one scrambles position while leaving
    // every vector well-formed.
    let device = Device::Cpu;
    let rope = Rope::new(4, 4, 1000.0, &device).unwrap();
    // x = [1, 0, 0, 0] at position 1: only the (0, 2) pair should move.
    let x = Tensor::from_vec(
        vec![0.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        (1, 1, 2, 4),
        &device,
    )
    .unwrap();

    let out = rope.apply(&x).unwrap().flatten_all().unwrap();
    let v = out.to_vec1::<f32>().unwrap();
    // Element 0 -> cos(1), element 2 -> sin(1). Element 1 stays zero, which
    // is what distinguishes this from the interleaved convention.
    assert!((v[4] - 1f32.cos()).abs() < 1e-5, "got {v:?}");
    assert!((v[6] - 1f32.sin()).abs() < 1e-5, "got {v:?}");
    assert!(v[5].abs() < 1e-6, "element 1 should be untouched: {v:?}");
}
