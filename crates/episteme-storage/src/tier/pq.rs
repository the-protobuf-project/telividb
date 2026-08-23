//! Scan tier over PQ codes.
//!
//! The tier where preparation pays for itself. Scoring a row is `m` table
//! lookups and `m` additions — no decode, no distance computation over the full
//! width — because the distances from the query to every centroid were computed
//! once, at prepare time.
//!
//! The table is **asymmetric**: the query stays full precision and only stored
//! vectors are quantized. Encoding the query too would add a second
//! quantization error for no saving, since the query is transformed once and
//! the corpus is scanned millions of times.

use crate::error::Result as StorageResult;
use crate::format::quantize::{CENTROIDS, PqCodebook};
use episteme_core::{Metric, Ordinal, PreparedQuery, PreparedState, Result, ScanTier};

/// PQ codes for one field, with the codebook that gives them meaning.
///
/// The codebook travels with the codes because a code is meaningless without
/// exactly the codebook that produced it.
#[derive(Debug)]
pub struct PqTier {
    codebook: PqCodebook,
    /// `m` bytes per present row; `None` where the field is absent.
    rows: Vec<Option<Vec<u8>>>,
}

impl PqTier {
    /// Encode every present row of `store` against `codebook`.
    pub fn build(
        store: &dyn episteme_core::VectorStore,
        codebook: PqCodebook,
    ) -> StorageResult<Self> {
        let mut rows = Vec::with_capacity(store.len());
        for row in 0..store.len() {
            let encoded = match store.get(Ordinal::from_row(row as u32)) {
                Some(v) => Some(codebook.encode(v)?),
                None => None,
            };
            rows.push(encoded);
        }
        Ok(Self { codebook, rows })
    }

    /// Parse a `codes.bin` written with this codec.
    pub fn from_codes(
        codes: &[u8],
        codebook: PqCodebook,
        rows: usize,
        is_present: &dyn Fn(usize) -> bool,
    ) -> StorageResult<Self> {
        let m = codebook.m();
        let parsed = (0..rows)
            .map(|row| {
                if !is_present(row) {
                    return Ok(None);
                }
                let start = row * m;
                codes
                    .get(start..start + m)
                    .map(|c| Some(c.to_vec()))
                    .ok_or(crate::error::Error::Truncated {
                        what: "pq codes",
                        needed: start + m,
                        found: codes.len(),
                    })
            })
            .collect::<StorageResult<Vec<_>>>()?;
        Ok(Self {
            codebook,
            rows: parsed,
        })
    }

    /// The codebook these codes were encoded against.
    pub fn codebook(&self) -> &PqCodebook {
        &self.codebook
    }

    /// Bytes the codes occupy, excluding the codebook itself.
    pub fn bytes(&self) -> usize {
        self.rows.iter().flatten().count() * self.codebook.m()
    }
}

impl ScanTier for PqTier {
    fn codec(&self) -> &'static str {
        "pq"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }

    fn prepare(&self, query: &[f32], metric: Metric) -> Result<PreparedQuery> {
        let dim = self.codebook.dim();
        if query.len() != dim {
            return Err(episteme_core::Error::DimMismatch {
                expected: dim,
                actual: query.len(),
            });
        }

        // One partial score per (subspace, centroid). Scoring a row then costs
        // m lookups rather than a full-width distance computation.
        let m = self.codebook.m();
        let sub_dim = dim / m;
        let mut distances = Vec::with_capacity(m * CENTROIDS);

        for sub in 0..m {
            let start = sub * sub_dim;
            let part = &query[start..start + sub_dim];
            for centroid in 0..CENTROIDS {
                let c = self.codebook.centroid(sub, centroid);
                distances.push(partial(metric, part, c));
            }
        }
        Ok(PreparedQuery::table(metric, m, distances))
    }

    fn score(&self, prepared: &PreparedQuery, ordinal: Ordinal) -> Option<f32> {
        let codes = self.rows.get(ordinal.row() as usize)?.as_ref()?;
        let PreparedState::Table {
            subspaces,
            distances,
        } = &prepared.state
        else {
            return None;
        };
        if codes.len() != *subspaces {
            return None;
        }

        // The sum of per-subspace partials. Exact for dot and L2, since both
        // decompose additively across disjoint subspaces — which is precisely
        // why product quantization works at all.
        let total: f32 = codes
            .iter()
            .enumerate()
            .map(|(sub, &code)| distances[sub * CENTROIDS + code as usize])
            .sum();
        Some(total)
    }
}

/// One subspace's contribution to the score.
///
/// Cosine is stored normalized and scored as dot, so it shares that arm.
fn partial(metric: Metric, query: &[f32], centroid: &[f32]) -> f32 {
    match metric {
        Metric::Dot | Metric::Cosine => query.iter().zip(centroid).map(|(a, b)| a * b).sum(),
        Metric::L2 => query
            .iter()
            .zip(centroid)
            .map(|(a, b)| (a - b) * (a - b))
            .sum(),
    }
}
