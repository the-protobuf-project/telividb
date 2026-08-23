//! Scan tier over half-precision rows.
//!
//! The mildest tier: half the bytes and effectively lossless for ranking, so it
//! needs the least over-fetch of any codec. Useful where memory is tight but
//! recall must not move at all.

use crate::error::{Error, Result as StorageResult};
use crate::format::quantize::F16Row;
use episteme_core::{Dim, Metric, Ordinal, PreparedQuery, PreparedState, Result, ScanTier};

/// Half-precision rows for one field.
#[derive(Debug)]
pub struct F16Tier {
    /// One entry per row; `None` where the field is absent.
    rows: Vec<Option<F16Row>>,
    /// Width of every row.
    dim: Dim,
}

impl F16Tier {
    /// Convert every present row of `store`.
    pub fn build(store: &dyn episteme_core::VectorStore) -> Self {
        let rows = (0..store.len())
            .map(|row| store.get(Ordinal::from_row(row as u32)).map(F16Row::encode))
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
    ) -> StorageResult<Self> {
        let row_bytes = F16Row::encoded_len(dim);
        let parsed = (0..rows)
            .map(|row| {
                if !is_present(row) {
                    return Ok(None);
                }
                let start = row * row_bytes;
                F16Row::read_from(&codes[start..], dim)
                    .map(Some)
                    .ok_or(Error::Truncated {
                        what: "f16 codes",
                        needed: start + row_bytes,
                        found: codes.len(),
                    })
            })
            .collect::<StorageResult<Vec<_>>>()?;
        Ok(Self {
            rows: parsed,
            dim: Dim::new(dim as u32)?,
        })
    }

    /// Bytes this tier occupies, for sizing decisions.
    pub fn bytes(&self) -> usize {
        self.rows.iter().flatten().count() * F16Row::encoded_len(self.dim.get())
    }
}

impl ScanTier for F16Tier {
    fn codec(&self) -> &'static str {
        "f16"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }

    fn prepare(&self, query: &[f32], metric: Metric) -> Result<PreparedQuery> {
        if query.len() != self.dim.get() {
            return Err(episteme_core::Error::DimMismatch {
                expected: self.dim.get(),
                actual: query.len(),
            });
        }
        Ok(PreparedQuery::vector(metric, query.to_vec()))
    }

    fn score(&self, prepared: &PreparedQuery, ordinal: Ordinal) -> Option<f32> {
        let row = self.rows.get(ordinal.row() as usize)?.as_ref()?;
        let PreparedState::Vector(query) = &prepared.state else {
            return None;
        };
        let mut decoded = vec![0f32; self.dim.get()];
        row.decode_into(&mut decoded);
        Some(episteme_distance::score(prepared.metric, query, &decoded))
    }
}
