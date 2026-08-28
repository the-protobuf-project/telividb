//! The ggml-backed inference server — layer four on layer one.
//!
//! Replaces the candle implementation entirely. The reason is not preference:
//! with layer one on ggml, keeping candle here would mean two tensor runtimes
//! resident, two GGUF loaders and two quantization implementations for one
//! model family. It also makes the encoder *simpler*, because ggml multiplies
//! a `Q4_K` weight against f32 activations natively — there is no
//! dequantization path to maintain.

mod attention;
mod batch;
mod block;
mod config;
mod encoder;
mod inferencer;
mod pipeline;
mod pool;
mod resident;
mod schedule;
mod vocab;
mod vocab_rules;

pub use config::{Config, SUPPORTED};
pub use encoder::Encoder;
pub use inferencer::GgmlInferencer;
pub use resident::ResidentModel;
