//! Building a device corpus straight from a store, with no file in between.
//!
//! **Why this exists.** `GpuFlatIndex::build` used to be `decode(encode(store))`
//! — it serialized the whole corpus into GGUF bytes in memory and immediately
//! parsed them back. That round trip is right for *persistence*, where rule 4
//! wants a versioned on-disk structure, and pointless when the caller only
//! wants an index in memory. On a million 128-dimension rows it was a 512 MB
//! serialize and re-parse for a result already sitting in the store.
//!
//! The written format is unchanged: [`super::gguf::write_corpus`] remains the
//! only thing that produces bytes, so what lands on disk is still versioned and
//! still the only thing `load_corpus` has to understand.

use super::gguf::{Corpus, candle_err};
use candle_core::{Device, Tensor};
use telividb_core::{Ordinal, Result, VectorStore};

/// Bytes a corpus of this shape will occupy on the device.
///
/// The budget must be reserved *before* the upload, because an over-large one
/// does not fail gracefully on Metal — it aborts the process — so this is the
/// last point at which the failure is still recoverable.
pub(super) fn device_bytes(store: &dyn VectorStore) -> usize {
    // The corpus tensor plus the presence column, both f32.
    store
        .len()
        .saturating_mul(store.dim().get().saturating_add(1))
        .saturating_mul(4)
}

/// Copy every present row of `store` onto `device`.
pub(super) fn corpus_from_store(store: &dyn VectorStore, device: &Device) -> Result<Corpus> {
    let dim = store.dim();
    let rows = store.len();
    let metric = store.metric();

    // An empty field is ordinary — a collection with no points yet, or a field
    // none of them populate. Built explicitly rather than falling through the
    // loop below, so the zero-row tensor is created once and deliberately.
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

    let width = dim.get();
    let mut flat = vec![0f32; rows * width];
    let mut present = Vec::with_capacity(rows);

    for row in 0..rows {
        match store.get(Ordinal::from_row(row as u32)) {
            Some(vector) => {
                flat[row * width..(row + 1) * width].copy_from_slice(vector);
                present.push(true);
            }
            // An absent row keeps its zeros. Against a dot product zero is a
            // real score rather than a neutral one, which is exactly why the
            // presence column is consulted during selection.
            None => present.push(false),
        }
    }

    Ok(Corpus {
        row_norms: std::sync::OnceLock::new(),
        tensor: Tensor::from_vec(flat, (rows, width), device).map_err(candle_err)?,
        present,
        dim,
        metric,
    })
}
