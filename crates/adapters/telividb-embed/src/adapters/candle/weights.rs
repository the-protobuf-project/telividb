//! Pulling tensors out of an open GGUF.
//!
//! Holds the file open for the duration of loading rather than reading the
//! whole thing into memory: a quantized encoder is hundreds of megabytes, and
//! `Content::tensor` seeks to exactly the block range it needs.

use crate::error::{Error, Result};
use candle_core::quantized::QTensor;
use candle_core::quantized::gguf_file::Content;
use candle_core::{Device, Tensor};
use std::fs::File;
use std::sync::Arc;

/// An open GGUF, positioned to serve tensors by name.
pub struct Weights {
    content: Content,
    file: File,
    device: Device,
}

impl Weights {
    /// Take ownership of an already-parsed GGUF and its reader.
    pub fn new(content: Content, file: File, device: Device) -> Self {
        Self {
            content,
            file,
            device,
        }
    }

    /// The parsed header, for reading metadata.
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Where tensors are being placed.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Fetch a tensor, left quantized.
    ///
    /// `Arc` because `QMatMul::from_arc` takes one and a shared weight is the
    /// normal case — cloning the blocks per layer would multiply the model's
    /// footprint by its layer count.
    pub fn quantized(&mut self, name: &str) -> Result<Arc<QTensor>> {
        let tensor = self
            .content
            .tensor(&mut self.file, name, &self.device)
            .map_err(|_| Error::MissingFromGguf {
                what: name.to_owned(),
            })?;
        Ok(Arc::new(tensor))
    }

    /// Fetch a tensor and dequantize it to f32.
    ///
    /// For the small ones that are used as values rather than as matrices:
    /// layer-norm scales and biases. These are one vector each, so the
    /// dequantized form costs nothing and avoids a `QMatMul` that would be
    /// wrong for them anyway.
    pub fn dequantized(&mut self, name: &str) -> Result<Tensor> {
        Ok(self.quantized(name)?.dequantize(&self.device)?)
    }

    /// Fetch a tensor that a model may legitimately not have.
    ///
    /// `None` rather than an error only where absence is meaningful — a
    /// projection stored without a bias. Everywhere else a missing tensor is
    /// [`Error::MissingFromGguf`], because silently substituting zeros would
    /// produce a model that runs and is wrong.
    pub fn optional(&mut self, name: &str) -> Option<Tensor> {
        self.dequantized(name).ok()
    }
}
