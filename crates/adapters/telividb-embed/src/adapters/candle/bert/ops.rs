//! Tensor operations the encoder needs that `candle-core` does not name.
//!
//! Written here rather than pulled from `candle-nn` on purpose: these are
//! three short functions over ops `candle-core` already has, and taking the
//! dependency to get them would pull `candle-nn` into every build for no
//! other reason.

use candle_core::{D, Result, Tensor};

/// Normalize over the last dimension, then scale and shift.
///
/// The biased (population) variance, matching what BERT was trained with.
/// `candle`'s `var_keepdim` applies Bessel's correction, which would divide by
/// `n - 1` and shift every activation slightly — small enough to look correct
/// and large enough to move neighbours.
pub fn layer_norm(xs: &Tensor, weight: &Tensor, bias: &Tensor, eps: f64) -> Result<Tensor> {
    let xs = xs.to_dtype(candle_core::DType::F32)?;
    let mean = xs.mean_keepdim(D::Minus1)?;
    let centered = xs.broadcast_sub(&mean)?;
    let variance = centered.sqr()?.mean_keepdim(D::Minus1)?;
    let normed = centered.broadcast_div(&(variance + eps)?.sqrt()?)?;
    normed.broadcast_mul(weight)?.broadcast_add(bias)
}

/// Softmax over the last dimension.
///
/// Subtracts the row maximum before exponentiating. Attention scores are
/// unbounded above, and `exp` of a large positive is `inf`, which propagates
/// to `NaN` through the division — the shift is what keeps the result finite.
pub fn softmax(xs: &Tensor) -> Result<Tensor> {
    let max = xs.max_keepdim(D::Minus1)?;
    let exp = xs.broadcast_sub(&max)?.exp()?;
    let sum = exp.sum_keepdim(D::Minus1)?;
    exp.broadcast_div(&sum)
}

/// The exact GELU, via the error function.
///
/// `Tensor::gelu` is the `tanh` approximation. BERT-family weights were
/// trained against the erf form, and the two differ by enough to move
/// rankings while leaving every vector well-formed.
pub fn gelu(xs: &Tensor) -> Result<Tensor> {
    xs.gelu_erf()
}

#[cfg(test)]
#[path = "ops_test.rs"]
mod tests;
