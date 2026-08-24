//! One vector field's corpus, as a GGUF tensor file.
//!
//! GGUF rather than a hand-rolled layout because it is already versioned,
//! self-describing and readable by `candle` straight onto a GPU device — which
//! satisfies invariant 4 through the format's own header instead of another
//! magic number this project would have to maintain. It sits beside the other
//! per-field artifacts as `vectors/<field>/index.gguf`, the same way
//! `index.hnsw` does.
//!
//! Two tensors are written, not one:
//!
//! - `corpus` — `(rows, dim)` f32, every row at its ordinal's position.
//! - `present` — `(rows,)` f32, `1.0` where the field has a value.
//!
//! An absent row still occupies its slot in `corpus`, written as zeros, so a
//! row's offset stays its ordinal (invariant 3's fixed stride, carried into
//! this format). `present` is what stops those zeros being scored as though
//! they were real vectors — a text-only point has no image vector, and zeros
//! against a dot product are not a neutral answer (invariant 17).

use candle_core::quantized::{GgmlDType, QTensor, gguf_file};
use candle_core::{Device, Tensor};
use telividb_core::{Dim, Metric, Ordinal, Result, VectorStore};

/// Metadata key holding the vector width.
const KEY_DIM: &str = "telividb.dim";
/// Metadata key holding the row count, absent rows included.
const KEY_ROWS: &str = "telividb.rows";
/// Metadata key holding the metric, so a file cannot be scored under the
/// wrong one after a schema edit.
const KEY_METRIC: &str = "telividb.metric";
/// Tensor holding the vectors themselves.
const TENSOR_CORPUS: &str = "corpus";
/// Tensor holding the presence mask.
const TENSOR_PRESENT: &str = "present";

/// A corpus loaded onto a device, with the facts needed to score it.
pub(super) struct Corpus {
    /// `(rows, dim)` on the device, ready to multiply against.
    pub tensor: Tensor,
    /// Host-side presence, one entry per row.
    ///
    /// Kept on the host rather than the device because it is consulted in the
    /// same CPU pass that selects top-k, alongside the visibility predicate —
    /// masking twice, once per side of the boundary, would buy nothing.
    pub present: Vec<bool>,
    /// Width of every row.
    pub dim: Dim,
    /// How these vectors are scored.
    pub metric: Metric,
}

/// Serialize every row of `store` into a GGUF file.
///
/// `GgmlDType::F32` deliberately: this must round-trip bit-exactly, because
/// the GPU index's correctness test is equality with the CPU flat index rather
/// than a recall threshold. Quantized types (Q8_0 and below) are the natural
/// follow-on and belong to the scan tier, not here.
pub(super) fn write_corpus<W: std::io::Write + std::io::Seek>(
    store: &dyn VectorStore,
    writer: &mut W,
) -> Result<()> {
    let dim = store.dim().get();
    let rows = store.len();

    let mut flat = vec![0f32; rows * dim];
    let mut present = vec![0f32; rows];
    for row in 0..rows {
        if let Some(vector) = store.get(Ordinal::from_row(row as u32)) {
            flat[row * dim..(row + 1) * dim].copy_from_slice(vector);
            present[row] = 1.0;
        }
    }

    // Built on the CPU regardless of where searching will happen: writing is a
    // one-time build step, and a device round trip here would buy nothing.
    let corpus = Tensor::from_vec(flat, (rows, dim), &Device::Cpu).map_err(candle_err)?;
    let mask = Tensor::from_vec(present, (rows,), &Device::Cpu).map_err(candle_err)?;
    let corpus = QTensor::quantize(&corpus, GgmlDType::F32).map_err(candle_err)?;
    let mask = QTensor::quantize(&mask, GgmlDType::F32).map_err(candle_err)?;

    let dim_value = gguf_file::Value::U32(dim as u32);
    let rows_value = gguf_file::Value::U64(rows as u64);
    let metric_value = gguf_file::Value::String(metric_name(store.metric()).to_owned());

    gguf_file::write(
        writer,
        &[
            (KEY_DIM, &dim_value),
            (KEY_ROWS, &rows_value),
            (KEY_METRIC, &metric_value),
        ],
        &[(TENSOR_CORPUS, &corpus), (TENSOR_PRESENT, &mask)],
    )
    .map_err(candle_err)
}

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
        tensor,
        present: present.into_iter().map(|p| p != 0.0).collect(),
        dim,
        metric,
    })
}

/// The metric's on-disk spelling.
///
/// Written out rather than stored as a discriminant so a renumbered enum
/// cannot silently reinterpret an existing file — the same reasoning invariant
/// 40 applies to protobuf field numbers.
fn metric_name(metric: Metric) -> &'static str {
    match metric {
        Metric::Dot => "dot",
        Metric::L2 => "l2",
        Metric::Cosine => "cosine",
    }
}

fn metric_of(name: &str) -> Result<Metric> {
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

/// Fold a candle failure into the domain error type.
///
/// By message, because `telividb-core` cannot depend on candle (invariant 14:
/// dependencies point inward) and so has no variant that could carry one.
fn candle_err(e: candle_core::Error) -> telividb_core::Error {
    telividb_core::Error::GpuIndex {
        reason: e.to_string(),
    }
}

#[cfg(test)]
#[path = "gguf_test.rs"]
mod tests;
