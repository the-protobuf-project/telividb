//! A quantized affine layer.

use candle_core::quantized::{QMatMul, QTensor};
use candle_core::{Module, Result, Tensor};
use std::sync::Arc;

/// `xs @ W^T + b`, with `W` left quantized.
///
/// The weight is never dequantized: `QMatMul` multiplies against the stored
/// blocks directly, which is the entire reason a GGUF model fits where its
/// f32 original would not. The bias stays f32 because it is one value per
/// output and quantizing it would cost accuracy for no meaningful space.
pub struct QLinear {
    weight: QMatMul,
    /// Absent for the rare projection stored without one, rather than a zero
    /// tensor — adding a zero costs a full-size broadcast per call.
    bias: Option<Tensor>,
}

impl QLinear {
    /// Build from a quantized weight and an optional f32 bias.
    pub fn new(weight: Arc<QTensor>, bias: Option<Tensor>) -> Result<Self> {
        Ok(Self {
            weight: QMatMul::from_arc(weight)?,
            bias,
        })
    }

    /// The layer's output width, when the bias reveals it.
    ///
    /// Read from the bias rather than the weight because a `QMatMul` does not
    /// expose its quantized shape. `None` for a bias-less projection, where
    /// the caller simply skips the check rather than guessing.
    pub fn output_width(&self) -> Option<usize> {
        self.bias.as_ref().map(|b| b.elem_count())
    }

    /// Apply to `xs`, whose last dimension must be the layer's input width.
    pub fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let out = self.weight.forward(xs)?;
        match &self.bias {
            Some(bias) => out.broadcast_add(bias),
            None => Ok(out),
        }
    }
}
