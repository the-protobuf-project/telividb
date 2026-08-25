//! Reading a corpus back off a GGUF into device memory.
//!
//! Split from `gguf.rs` so that file is the writer and this is the reader.
//! They share a format and nothing else — and the reader is where the
//! defensive work lives, because the bytes it parses may have been written by
//! a different build.

use super::gguf::{
    Corpus, KEY_DIM, KEY_METRIC, KEY_ROWS, TENSOR_CORPUS, TENSOR_PRESENT, candle_err,
};
use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use telividb_core::{Dim, Metric, Result};

/// Read a corpus back, placing the vectors on `device`.
pub(super) fn load_corpus<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    device: &Device,
) -> Result<Corpus> {
    let content = gguf_file::Content::read(reader).map_err(candle_err)?;

    let dim = match content.metadata.get(KEY_DIM) {
        Some(gguf_file::Value::U32(dim)) => Dim::new(*dim)?,
        _ => return Err(malformed("missing or non-u32 telividb.dim")),
    };
    let metric = match content.metadata.get(KEY_METRIC) {
        Some(gguf_file::Value::String(name)) => metric_of(name)?,
        _ => return Err(malformed("missing or non-string telividb.metric")),
    };

    let rows = match content.metadata.get(KEY_ROWS) {
        Some(gguf_file::Value::U64(rows)) => *rows as usize,
        _ => return Err(malformed("missing or non-u64 telividb.rows")),
    };

    // An empty field is ordinary — a collection with no points yet, or a
    // field none of them populate — but candle's GGUF reader dereferences a
    // null pointer on a zero-element tensor and *aborts the process* rather
    // than returning an error, so its tensors are never asked for here. The
    // row count in the metadata is enough to reconstruct an empty corpus.
    if rows == 0 {
        return Ok(Corpus {
            row_norms: std::sync::OnceLock::new(),
            tensor: Tensor::zeros((0, dim.get()), candle_core::DType::F32, device)
                .map_err(candle_err)?,
            present: Vec::new(),
            dim,
            metric,
        });
    }

    let tensor = content
        .tensor(reader, TENSOR_CORPUS, device)
        .map_err(candle_err)?
        .dequantize(device)
        .map_err(candle_err)?;

    // Presence is pulled to the host: it is read once per row during top-k
    // selection, which already runs there.
    let present: Vec<f32> = content
        .tensor(reader, TENSOR_PRESENT, &Device::Cpu)
        .map_err(candle_err)?
        .dequantize(&Device::Cpu)
        .map_err(candle_err)?
        .flatten_all()
        .map_err(candle_err)?
        .to_vec1()
        .map_err(candle_err)?;

    Ok(Corpus {
        row_norms: std::sync::OnceLock::new(),
        tensor,
        present: present.into_iter().map(|p| p != 0.0).collect(),
        dim,
        metric,
    })
}

pub(super) fn metric_of(name: &str) -> Result<Metric> {
    match name {
        "dot" => Ok(Metric::Dot),
        "l2" => Ok(Metric::L2),
        "cosine" => Ok(Metric::Cosine),
        _ => Err(malformed("unknown telividb.metric")),
    }
}

fn malformed(reason: &'static str) -> telividb_core::Error {
    telividb_core::Error::MalformedIndex { reason }
}
