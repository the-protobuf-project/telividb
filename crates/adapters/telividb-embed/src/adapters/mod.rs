//! Implementations of the inference boundary.
//!
//! One today, and the port is what keeps that a fact rather than a ceiling.
//! `ggml` backs every model because layer one already binds it: a second
//! *tensor runtime* here would mean two GGUF loaders, two quantization
//! implementations and two hardware-backend surfaces for the same model family
//! (rule 42).
//!
//! A second **model runtime** above the engine is a different matter and is
//! explicitly permitted — ONNX through `ort` is the provision, for
//! architectures no GGUF loader reaches. It would be a sibling of `ggml` here,
//! implementing the same [`Inferencer`] port, and it would not touch any layer
//! below. Nothing in this module's shape needs to change to admit it, which is
//! the point of having a port at all.
//!
//! [`Inferencer`]: crate::ports::Inferencer

pub mod ggml;

pub use ggml::GgmlInferencer;
