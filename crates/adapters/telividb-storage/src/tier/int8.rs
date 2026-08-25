//! Scan tier over int8 codes.
//!
//! Decodes a row before scoring it, because the per-row scale and offset mean
//! codes from different rows are not directly comparable. That still beats full
//! precision: the decode reads a quarter as many bytes, and memory bandwidth is
//! what a scan is actually bound by.

use crate::format::quantize::Int8Row;
use telividb_core::{Dim, Metric, Ordinal, PreparedQuery, PreparedState, Result, ScanTier};
use telividb_distance::Scorer;

/// Int8-quantized rows for one field.
#[derive(Debug)]
pub struct Int8Tier {
    rows: Vec<Option<Int8Row>>,
    dim: Dim,
}

impl Int8Tier {
    /// Quantize every present row of `store`.
    pub fn build(store: &dyn telividb_core::VectorStore) -> Self {
        let rows = (0..store.len())
            .map(|row| {
                store
                    .get(Ordinal::from_row(row as u32))
                    .map(Int8Row::encode)
            })
            .collect();
        Self {
            rows,
            dim: store.dim(),
        }
    }

    /// Parse a `codes.bin` written with this codec.
    pub fn from_codes(
        codes: &[u8],
        dim: usize,
        rows: usize,
        is_present: &dyn Fn(usize) -> bool,
    ) -> crate::error::Result<Self> {
        let row_bytes = Int8Row::encoded_len(dim);
        let parsed = (0..rows)
            .map(|row| {
                if !is_present(row) {
                    return Ok(None);
                }
                // `codes.get(..)` rather than `codes[..]`: this is a public
                // function taking untrusted bytes, and the slice panicked
                // before the `Truncated` error below could ever be built.
                let start = row * row_bytes;
                codes
                    .get(start..)
                    .and_then(|rest| Int8Row::read_from(rest, dim))
                    .map(Some)
                    .ok_or(crate::error::Error::Truncated {
                        what: "int8 codes",
                        needed: start + row_bytes,
                        found: codes.len(),
                    })
            })
            .collect::<crate::error::Result<Vec<_>>>()?;
        Ok(Self {
            rows: parsed,
            dim: Dim::new(dim as u32)?,
        })
    }

    /// Bytes this tier occupies, for sizing decisions.
    pub fn bytes(&self) -> usize {
        self.rows.iter().flatten().count() * Int8Row::encoded_len(self.dim.get())
    }
}

impl ScanTier for Int8Tier {
    fn codec(&self) -> &'static str {
        "int8"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }

    fn prepare(&self, query: &[f32], metric: Metric) -> Result<PreparedQuery> {
        if query.len() != self.dim.get() {
            return Err(telividb_core::Error::DimMismatch {
                expected: self.dim.get(),
                actual: query.len(),
            });
        }
        // The query stays full precision — quantizing both sides would compound
        // the error for no saving, since the query is transformed once.
        Ok(PreparedQuery::vector(metric, query.to_vec()))
    }

    fn score(&self, prepared: &PreparedQuery, ordinal: Ordinal) -> Option<f32> {
        let row = self.rows.get(ordinal.row() as usize)?.as_ref()?;
        let PreparedState::Vector(query) = &prepared.state else {
            return None;
        };
        let mut decoded = vec![0f32; self.dim.get()];
        row.decode_into(&mut decoded);
        Some(prepared.metric.score(query, &decoded))
    }
}

#[cfg(test)]
#[path = "int8_test.rs"]
mod tests;
