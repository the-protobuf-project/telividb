use super::*;
use candle_core::{Device, Tensor};

#[test]
fn layer_norm_uses_the_biased_variance_bert_was_trained_with() {
    // Bessel's correction would divide by n-1 here, giving a visibly larger
    // spread. The distinction is invisible in a smoke test and shifts every
    // activation in a real one.
    let device = Device::Cpu;
    let xs = Tensor::from_vec(vec![1.0f32, 2.0, 3.0, 4.0], (1, 4), &device).unwrap();
    let weight = Tensor::ones((4,), candle_core::DType::F32, &device).unwrap();
    let bias = Tensor::zeros((4,), candle_core::DType::F32, &device).unwrap();

    let out = layer_norm(&xs, &weight, &bias, 0.0).unwrap();
    let values = out.flatten_all().unwrap().to_vec1::<f32>().unwrap();

    // mean 2.5, population sd = sqrt(1.25) ≈ 1.1180.
    let expected = [-1.3416, -0.4472, 0.4472, 1.3416];
    for (got, want) in values.iter().zip(expected) {
        assert!((got - want).abs() < 1e-3, "got {values:?}");
    }
}

#[test]
fn softmax_survives_a_large_score_instead_of_returning_nan() {
    // Attention scores are unbounded above. Without the max-subtraction, exp
    // overflows to inf and the division yields NaN — which then propagates
    // silently through every downstream layer.
    let device = Device::Cpu;
    let xs = Tensor::from_vec(vec![1000.0f32, 1.0, 2.0], (1, 3), &device).unwrap();

    let out = softmax(&xs).unwrap().flatten_all().unwrap();
    let values = out.to_vec1::<f32>().unwrap();

    assert!(values.iter().all(|v| v.is_finite()), "got {values:?}");
    assert!((values.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    assert!(values[0] > 0.99, "the large score should dominate");
}

#[test]
fn softmax_rows_each_sum_to_one() {
    let device = Device::Cpu;
    let xs = Tensor::from_vec(vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0], (2, 3), &device).unwrap();
    let out = softmax(&xs).unwrap().sum_keepdim(D::Minus1).unwrap();
    for value in out.flatten_all().unwrap().to_vec1::<f32>().unwrap() {
        assert!((value - 1.0).abs() < 1e-5);
    }
}

#[test]
fn gelu_is_the_erf_form_not_the_tanh_approximation() {
    // The two agree closely near zero and diverge in the tails, which is
    // exactly where they would quietly move a ranking.
    let device = Device::Cpu;
    let xs = Tensor::from_vec(vec![-3.0f32, 3.0], (2,), &device).unwrap();

    let exact = gelu(&xs).unwrap().to_vec1::<f32>().unwrap();
    let approx = xs.gelu().unwrap().to_vec1::<f32>().unwrap();

    assert!(
        (exact[0] - approx[0]).abs() > 1e-6 || (exact[1] - approx[1]).abs() > 1e-6,
        "erf and tanh GELU should differ in the tails; got {exact:?} vs {approx:?}"
    );
    // erf-GELU(3) = 3 * Φ(3) ≈ 2.9960
    assert!((exact[1] - 2.9960).abs() < 1e-3, "got {exact:?}");
}
