//! Rows accepted but not yet sealed into a segment.
//!
//! **The buffer is searchable, and that is a Phase 1 decision rather than a
//! later optimization.** Every query scans it exhaustively and merges those
//! hits with the sealed-segment results before top-k selection. Without that,
//! a write is invisible until enough rows accumulate to trip the seal
//! threshold — fine for bulk import, unacceptable for interactive or streaming
//! ingest, where "I just wrote it and cannot find it" is the first thing anyone
//! notices.
//!
//! It changes the planner and the merge step, which is why it cannot be bolted
//! on afterwards.
//!
//! The scan is **exact**, so buffer hits can only improve recall. But recall
//! accounting must still separate them from index hits, or a measurement of
//! approximate-index quality silently includes exhaustive results and reads as
//! noise.

use episteme_core::{Dim, Error, Metric, Ordinal, Result, VectorStore};

/// Vectors held in memory, in one contiguous allocation, awaiting seal.
#[derive(Debug, Clone)]
pub struct MutableBuffer {
    data: Vec<f32>,
    present: Vec<bool>,
    dim: Dim,
    metric: Metric,
}

impl MutableBuffer {
    pub fn new(dim: Dim, metric: Metric) -> Self {
        Self {
            data: Vec::new(),
            present: Vec::new(),
            dim,
            metric,
        }
    }

    /// Pre-allocate for `rows`, avoiding reallocation as the buffer fills
    /// toward its seal threshold.
    pub fn with_capacity(dim: Dim, metric: Metric, rows: usize) -> Self {
        Self {
            data: Vec::with_capacity(rows * dim.get()),
            present: Vec::with_capacity(rows),
            dim,
            metric,
        }
    }

    /// Append a vector, normalising first when the metric requires it.
    ///
    /// Rejects a non-finite component rather than storing it: a NaN poisons
    /// every later comparison, and the resulting ordering is inconsistent in a
    /// way that surfaces far from the write that caused it.
    pub fn push(&mut self, vector: &[f32]) -> Result<Ordinal> {
        if vector.len() != self.dim.get() {
            return Err(Error::DimMismatch {
                expected: self.dim.get(),
                actual: vector.len(),
            });
        }
        if let Some(index) = vector.iter().position(|x| !x.is_finite()) {
            return Err(Error::NonFinite { index });
        }

        let start = self.data.len();
        self.data.extend_from_slice(vector);
        if self.metric.normalises_at_ingest() {
            episteme_distance::normalize(&mut self.data[start..]);
        }
        self.present.push(true);
        Ok(Ordinal::from_row((self.present.len() - 1) as u32))
    }

    /// Append a row carrying no value for this field.
    ///
    /// Normal in a multimodal collection — a text-only point has no image
    /// vector — and recorded explicitly rather than faked with zeros, which
    /// would otherwise be scored like any other vector.
    pub fn push_absent(&mut self) -> Ordinal {
        self.data.resize(self.data.len() + self.dim.get(), 0.0);
        self.present.push(false);
        Ordinal::from_row((self.present.len() - 1) as u32)
    }

    /// Rows currently buffered, present or absent.
    pub fn rows(&self) -> usize {
        self.present.len()
    }

    /// Heap bytes held. Drives the seal threshold.
    pub fn bytes(&self) -> usize {
        self.data.len() * std::mem::size_of::<f32>() + self.present.len()
    }

    /// Whether the buffer has reached the size at which it should be sealed.
    pub fn should_seal(&self, threshold_bytes: usize) -> bool {
        self.bytes() >= threshold_bytes
    }

    /// Discard every row, keeping the allocation for the next batch.
    ///
    /// Called after a successful seal — never before the segment is durable and
    /// the manifest names it, or the rows vanish from both places at once.
    pub fn clear(&mut self) {
        self.data.clear();
        self.present.clear();
    }
}

impl VectorStore for MutableBuffer {
    fn dim(&self) -> Dim {
        self.dim
    }

    fn metric(&self) -> Metric {
        self.metric
    }

    fn len(&self) -> usize {
        self.present.len()
    }

    fn get(&self, ordinal: Ordinal) -> Option<&[f32]> {
        let row = ordinal.row() as usize;
        if !*self.present.get(row)? {
            return None;
        }
        let start = row * self.dim.get();
        self.data.get(start..start + self.dim.get())
    }
}

#[cfg(test)]
#[path = "mutable_test.rs"]
mod tests;
