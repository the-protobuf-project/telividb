//! Implementations of the inference boundary.
//!
//! Exactly one, permanently (rule 42). A second local runtime — ONNX,
//! TensorRT, OpenVINO — is not a fallback to be added later; it is a second
//! C++ dependency tree and a second hardware-backend surface, both of which
//! this project rejected. A model with no candle path is out of scope for
//! local inference.

pub mod candle;

pub use candle::CandleInferencer;
