//! Scan tier over binary codes.
//!
//! The only tier that scores **without decoding anything**: both sides become
//! sign bits and the comparison is a popcount. That makes it the cheapest
//! possible prune, and also the coarsest — it exists to cut millions of
//! candidates to thousands, never to rank the answer.

use crate::format::quantize::{BinaryCodes, hamming};
use episteme_core::{Dim, Metric, Ordinal, PreparedQuery, PreparedState, Result, ScanTier};

/// Sign-bit codes for one field.
#[derive(Debug)]
pub struct BinaryTier {
    rows: Vec<Option<BinaryCodes>>,
    dim: Dim,
}

impl BinaryTier {
    /// Pack sign bits for every present row of `store`.
    pub fn build(store: &dyn episteme_core::VectorStore) -> Self {
        let rows = (0..store.len())
            .map(|row| {
                store
                    .get(Ordinal::from_row(row as u32))
                    .map(BinaryCodes::encode)
            })
            .collect();
        Self {
            rows,
            dim: store.dim(),
        }
    }

    /// Bytes this tier occupies, for sizing decisions.
    pub fn bytes(&self) -> usize {
        self.rows.iter().flatten().count() * BinaryCodes::encoded_len(self.dim.get())
    }
}

impl ScanTier for BinaryTier {
    fn codec(&self) -> &'static str {
        "binary"
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
        Ok(PreparedQuery::codes(
            metric,
            BinaryCodes::encode(query).as_bytes().to_vec(),
        ))
    }

    fn score(&self, prepared: &PreparedQuery, ordinal: Ordinal) -> Option<f32> {
        let row = self.rows.get(ordinal.row() as usize)?.as_ref()?;
        let PreparedState::Codes(bytes) = &prepared.state else {
            return None;
        };
        let query = BinaryCodes::from_bytes(bytes, self.dim.get())?;
        let distance = hamming(&query, row)? as f32;

        // Hamming is a distance, but callers compare scores on the metric's own
        // scale. Negate for metrics where higher means nearer so the ordering
        // matches, rather than making every caller special-case this tier.
        Some(if prepared.metric.higher_is_nearer() {
            -distance
        } else {
            distance
        })
    }
}
