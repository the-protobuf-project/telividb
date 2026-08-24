//! Deterministic weights for the fixture model.

use candle_core::quantized::{GgmlDType, QTensor};
use candle_core::{Device, Result, Tensor};

/// Fill a tensor with small, distinct, reproducible values.
///
/// Deterministic rather than random so a failure is the same failure on the
/// next run. Values are kept small and non-uniform: all-equal weights make a
/// transposition bug invisible, because the wrong element has the same value
/// as the right one.
pub fn filled(shape: (usize, usize), seed: f32, device: &Device) -> Result<Tensor> {
    let (rows, cols) = shape;
    let data: Vec<f32> = (0..rows * cols)
        .map(|i| ((i as f32 * 0.37 + seed).sin()) * 0.1)
        .collect();
    Tensor::from_vec(data, shape, device)
}

/// The same, as a one-dimensional bias or norm vector.
pub fn vector(len: usize, seed: f32, device: &Device) -> Result<Tensor> {
    let data: Vec<f32> = (0..len)
        .map(|i| ((i as f32 * 0.53 + seed).cos()) * 0.1)
        .collect();
    Tensor::from_vec(data, (len,), device)
}

/// Quantize to F32 blocks.
///
/// F32 rather than a real quantization because the fixture tests *loading and
/// shape*, not numerical fidelity. A Q4 fixture would add rounding noise to
/// every assertion for no extra coverage — the `QMatMul` path is identical
/// either way, and candle's own tests cover the block formats.
pub fn quantized(tensor: &Tensor) -> Result<QTensor> {
    QTensor::quantize(tensor, GgmlDType::F32)
}
