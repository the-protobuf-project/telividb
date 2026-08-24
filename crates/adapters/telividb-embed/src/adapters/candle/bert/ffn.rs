//! The feed-forward half of a block, in either of its two shapes.

use super::linear::QLinear;
use super::ops::gelu;
use crate::adapters::candle::weights::Weights;
use crate::error::Result;
use candle_core::Tensor;

/// How a model's feed-forward network is built.
///
/// Chosen from which tensors the file carries, not from the architecture name.
/// The two compute genuinely different functions, and running one model's
/// weights through the other's arithmetic produces finite, well-shaped,
/// meaningless vectors.
pub enum FeedForward {
    /// Classic BERT: `down(gelu(up(x)))`.
    Plain {
        /// Projection up to the intermediate width.
        up: QLinear,
        /// Projection back down to the hidden width.
        down: QLinear,
    },
    /// nomic-bert's SwiGLU: `down(up(x) * silu(gate(x)))`.
    ///
    /// Note where the activation sits — on the *gate* branch only, with the
    /// value branch passed through unactivated. Applying it to both, or to the
    /// wrong one, is a silent accuracy loss rather than an error.
    Gated {
        /// Value branch, unactivated.
        up: QLinear,
        /// Gate branch, passed through SiLU.
        gate: QLinear,
        /// Projection back down to the hidden width.
        down: QLinear,
    },
}

impl FeedForward {
    /// Load block `i`'s feed-forward weights, gated when a gate is present.
    pub fn load(weights: &mut Weights, i: usize) -> Result<Self> {
        let up = load_linear(weights, &format!("blk.{i}.ffn_up"))?;
        let down = load_linear(weights, &format!("blk.{i}.ffn_down"))?;

        match load_linear(weights, &format!("blk.{i}.ffn_gate")) {
            Ok(gate) => Ok(FeedForward::Gated { up, gate, down }),
            Err(_) => Ok(FeedForward::Plain { up, down }),
        }
    }

    /// Run `xs` through the network.
    pub fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        match self {
            FeedForward::Plain { up, down } => Ok(down.forward(&gelu(&up.forward(xs)?)?)?),
            FeedForward::Gated { up, gate, down } => {
                let value = up.forward(xs)?;
                let gated = gate.forward(xs)?.silu()?;
                Ok(down.forward(&(value * gated)?)?)
            }
        }
    }

    /// The intermediate width, when a bias reveals it.
    pub fn intermediate_width(&self) -> Option<usize> {
        match self {
            FeedForward::Plain { up, .. } | FeedForward::Gated { up, .. } => up.output_width(),
        }
    }
}

/// Load a `<prefix>.weight` / optional `<prefix>.bias` pair.
fn load_linear(weights: &mut Weights, prefix: &str) -> Result<QLinear> {
    let weight = weights.quantized(&format!("{prefix}.weight"))?;
    let bias = weights.optional(&format!("{prefix}.bias"));
    Ok(QLinear::new(weight, bias)?)
}
