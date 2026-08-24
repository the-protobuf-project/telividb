//! An in-memory [`VectorStore`], for tests and small collections.
//!
//! Rudimentary by design: the mmap-backed segment store replaces this behind
//! the same port without any index change.

use crate::ports::VectorStore;
use telividb_core::{Dim, Metric, Ordinal};

/// Row-major vectors held in one contiguous allocation.
#[derive(Debug, Clone)]
pub struct MemoryStore {
    data: Vec<f32>,
    present: Vec<bool>,
    dim: Dim,
    metric: Metric,
}

impl MemoryStore {
    /// An empty store for vectors of `dim` width scored by `metric`.
    pub fn new(dim: Dim, metric: Metric) -> Self {
        Self {
            data: Vec::new(),
            present: Vec::new(),
            dim,
            metric,
        }
    }

    /// Append a vector, normalising first when the metric requires it.
    pub fn push(&mut self, vector: &[f32]) -> telividb_core::Result<Ordinal> {
        if vector.len() != self.dim.get() {
            return Err(telividb_core::Error::DimMismatch {
                expected: self.dim.get(),
                actual: vector.len(),
            });
        }
        if let Some(index) = vector.iter().position(|x| !x.is_finite()) {
            return Err(telividb_core::Error::NonFinite { index });
        }

        let start = self.data.len();
        self.data.extend_from_slice(vector);
        if self.metric.normalises_at_ingest() {
            telividb_distance::normalize(&mut self.data[start..]);
        }
        self.present.push(true);
        Ok(Ordinal::from_row((self.present.len() - 1) as u32))
    }

    /// Append a row that has no value for this field.
    pub fn push_absent(&mut self) -> Ordinal {
        self.data.resize(self.data.len() + self.dim.get(), 0.0);
        self.present.push(false);
        Ordinal::from_row((self.present.len() - 1) as u32)
    }
}

impl VectorStore for MemoryStore {
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
