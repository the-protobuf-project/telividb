//! A quantized BERT encoder, written against GGUF directly.
//!
//! **Why this exists rather than `candle_transformers::models::bert`.** That
//! model takes the safetensors `VarBuilder`; there is no GGUF-backed embedding
//! model anywhere in candle's zoo. Loading safetensors instead would break
//! rule 12's identity story, where a model *is* the SHA-256 of its GGUF — so
//! the encoder is written here against `QTensor` and `QMatMul`.
//!
//! It is the whole forward pass and no more: embeddings, N attention blocks,
//! pooling. No decoder, no cross-attention, no cache — an embedding model runs
//! once over a full sequence and never generates.

mod attention;
mod block;
mod embeddings;
mod encode;
mod ffn;
mod linear;
mod ops;
mod qkv;
mod rope;

pub use encode::QuantizedBert;
