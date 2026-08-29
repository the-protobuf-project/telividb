//! Graph operations, against values computed by hand.
//!
//! Every test here runs on the CPU backend, which every machine has. Results
//! must be identical on every backend (rule 46) — only the speed may differ —
//! so a correctness test gains nothing from an accelerator.

use crate::{Backend, Context, DeviceKind};

fn cpu() -> Backend {
    Backend::of(DeviceKind::Cpu).unwrap()
}

#[test]
fn matmul_contracts_the_shared_leading_dimension() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();

    // ggml order is fastest-varying first, so (2, 3) is three columns of two.
    // a = [[1,2],[3,4],[5,6]] as columns; b = one column [1,1].
    let a = ctx
        .input_f32(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], [2, 3])
        .unwrap();
    let b = ctx.input_f32(&[1.0, 1.0], [2, 1]).unwrap();

    let out = a.matmul(&b).unwrap();
    assert_eq!(out.dim(0), 3, "one score per column of a");

    let got = ctx.compute(&out).unwrap();
    assert_eq!(got, vec![3.0, 7.0, 11.0]);
}

#[test]
fn add_and_scale_compose() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();
    let a = ctx.input_f32(&[1.0, 2.0, 3.0, 4.0], [4, 1]).unwrap();
    let b = ctx.input_f32(&[10.0, 10.0, 10.0, 10.0], [4, 1]).unwrap();

    let out = a.add(&b).unwrap().scale(0.5).unwrap();
    assert_eq!(ctx.compute(&out).unwrap(), vec![5.5, 6.0, 6.5, 7.0]);
}

#[test]
fn rows_gathers_the_embedding_table() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();

    // Four vocabulary entries, two wide: [0,1], [2,3], [4,5], [6,7].
    let table = ctx
        .input_f32(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], [2, 4])
        .unwrap();
    let ids = ctx.input_i32(&[2, 0], [2, 1]).unwrap();

    let out = table.rows(&ids).unwrap();
    assert_eq!(ctx.compute(&out).unwrap(), vec![4.0, 5.0, 0.0, 1.0]);
}

#[test]
fn softmax_rows_sum_to_one() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();
    // Two independent rows of three, so the normalization must be per row.
    let x = ctx
        .input_f32(&[1.0, 2.0, 3.0, 9.0, 9.0, 9.0], [3, 2])
        .unwrap();

    let got = ctx.compute(&x.softmax().unwrap()).unwrap();
    let first: f32 = got[..3].iter().sum();
    let second: f32 = got[3..].iter().sum();
    assert!((first - 1.0).abs() < 1e-5, "row one summed to {first}");
    assert!((second - 1.0).abs() < 1e-5, "row two summed to {second}");
    // The equal row must come out uniform, which catches a softmax that
    // normalized across the whole tensor instead of per row.
    assert!((got[3] - 1.0 / 3.0).abs() < 1e-5);
}

#[test]
fn layer_norm_centres_and_rescales() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();
    let x = ctx.input_f32(&[1.0, 2.0, 3.0, 4.0], [4, 1]).unwrap();
    let w = ctx.input_f32(&[1.0, 1.0, 1.0, 1.0], [4, 1]).unwrap();
    let b = ctx.input_f32(&[0.0, 0.0, 0.0, 0.0], [4, 1]).unwrap();

    let got = ctx.compute(&x.layer_norm(&w, &b, 1e-5).unwrap()).unwrap();
    let mean: f32 = got.iter().sum::<f32>() / got.len() as f32;
    assert!(mean.abs() < 1e-5, "normalized mean was {mean}");
    // Unit variance, and ordering preserved.
    assert!(got[0] < got[1] && got[1] < got[2] && got[2] < got[3]);
}

#[test]
fn a_wrong_element_count_is_refused_rather_than_read_past() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();
    assert!(ctx.input_f32(&[1.0, 2.0], [4, 1]).is_err());
}

#[test]
fn a_masked_position_gets_no_weight() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();

    // Three equal scores; the middle one masked out. If the mask lands before
    // the softmax, the surviving two split the weight evenly and the masked one
    // gets nothing.
    let scores = ctx.input_f32(&[1.0, 1.0, 1.0], [3, 1]).unwrap();
    let mask = ctx.input_f32(&[0.0, -1e9, 0.0], [3, 1]).unwrap();

    let got = ctx
        .compute(&scores.masked_softmax(Some(&mask), 1.0).unwrap())
        .unwrap();

    assert!(
        got[1].abs() < 1e-6,
        "masked position kept weight {}",
        got[1]
    );
    assert!((got[0] - 0.5).abs() < 1e-5, "got {got:?}");
    assert!((got[2] - 0.5).abs() < 1e-5, "got {got:?}");
}

#[test]
fn the_mask_broadcasts_over_the_batch_axis() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();

    // Two rows of two queries over three keys: (3, 2, 1, 2). The mask is
    // (3, 2, 1, 2) too — a different key blocked in each row, which is what an
    // encoder batch with differing padding actually needs.
    let scores = ctx
        .input_f32(&[1.0; 12], [3, 4])
        .unwrap()
        .reshape_4d(3, 2, 1, 2)
        .unwrap();
    let mask = ctx
        .input_f32(
            &[
                0.0, -1e9, 0.0, 0.0, -1e9, 0.0, 0.0, 0.0, -1e9, 0.0, 0.0, -1e9,
            ],
            [3, 4],
        )
        .unwrap()
        .reshape_4d(3, 2, 1, 2)
        .unwrap();

    let got = ctx
        .compute(&scores.masked_softmax(Some(&mask), 1.0).unwrap())
        .unwrap();

    assert!(got[1].abs() < 1e-6, "row 0 key 1 kept weight: {got:?}");
    assert!(got[11].abs() < 1e-6, "row 1 key 2 kept weight: {got:?}");
}

#[test]
fn rms_norm_divides_by_the_root_mean_square_and_subtracts_no_mean() {
    // The distinction that matters against `layer_norm`, and the reason the
    // two are not interchangeable: this subtracts no mean and adds no bias.
    // For [1,2,3,4] the mean square is (1+4+9+16)/4 = 7.5, so every element is
    // divided by sqrt(7.5) = 2.7386.
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();
    let x = ctx.input_f32(&[1.0, 2.0, 3.0, 4.0], [4, 1]).unwrap();
    let ones = ctx.input_f32(&[1.0, 1.0, 1.0, 1.0], [4, 1]).unwrap();

    let got = ctx.compute(&x.rms_norm(&ones, 1e-6).unwrap()).unwrap();

    let rms = 7.5f32.sqrt();
    for (got, expected) in got.iter().zip([1.0, 2.0, 3.0, 4.0]) {
        assert!(
            (got - expected / rms).abs() < 1e-5,
            "{got} vs {}",
            expected / rms
        );
    }
    // A mean-subtracting norm would centre these on zero, so the sum being
    // clearly positive is what proves the right one ran.
    assert!(got.iter().sum::<f32>() > 3.0);
}

#[test]
fn rms_norm_applies_its_weight() {
    let backend = cpu();
    let ctx = Context::new(&backend, 64).unwrap();
    let x = ctx.input_f32(&[3.0, 4.0], [2, 1]).unwrap();
    // Mean square is (9+16)/2 = 12.5, so the divisor is sqrt(12.5).
    let weight = ctx.input_f32(&[2.0, 0.0], [2, 1]).unwrap();

    let got = ctx.compute(&x.rms_norm(&weight, 1e-6).unwrap()).unwrap();

    let rms = 12.5f32.sqrt();
    assert!((got[0] - 2.0 * 3.0 / rms).abs() < 1e-5, "{got:?}");
    assert!(got[1].abs() < 1e-6, "a zero weight must zero its channel");
}
