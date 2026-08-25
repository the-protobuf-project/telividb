//! IVF with product-quantized residuals.
//!
//! IVF cuts *how many* rows are scored; PQ cuts *what each row costs* to score
//! and to store. Together they are what lets a corpus far larger than memory be
//! searched at all.
//!
//! **Residuals, not absolute vectors.** Each row is encoded relative to its
//! own list's centroid. The values being quantized are then small and centred
//! on zero rather than spread across the whole space, so the same code size
//! carries far more information — that is most of why IVF-PQ beats PQ alone.
//! Encoding absolutely would still *work*, and would quietly lose accuracy
//! that no test distinguishes from a poorly-trained codebook.

use super::params::IvfParams;
use super::quantizer::Coarse;
use std::time::Instant;
use telividb_core::{Ordinal, Result, VectorStore};
use telividb_distance::pq::{PqCodebook, PqParams};
use telividb_telemetry::{Meter, fields, logger};

/// One list's rows and their codes, laid out contiguously.
pub(super) struct List {
    /// Row numbers, parallel to the code runs.
    pub(super) rows: Vec<u32>,
    /// `rows.len() * m` codes, row-major.
    pub(super) codes: Vec<u8>,
}

/// An IVF index whose rows are stored as product-quantized residuals.
pub struct IvfPqIndex {
    pub(super) coarse: Coarse,
    pub(super) codebook: PqCodebook,
    pub(super) lists: Vec<List>,
    pub(super) params: IvfParams,
    /// Candidates rescored exactly, as a multiple of `k`.
    pub(super) rescore: usize,
    pub(super) meter: Meter,
}

impl IvfPqIndex {
    /// Train both quantizers over `store` and encode every present row.
    pub fn build(store: &dyn VectorStore, params: IvfParams, pq: PqParams) -> Result<Self> {
        let started = Instant::now();
        let coarse = Coarse::train(store, &params)?;

        // Residuals, so the codebook is trained on what it will actually
        // encode. Training on absolute vectors and encoding residuals would
        // put every code in a region of the space the codebook never saw.
        let residuals = sample_residuals(store, &coarse, &params);
        let borrowed: Vec<&[f32]> = residuals.iter().map(Vec::as_slice).collect();
        let codebook = PqCodebook::train(&borrowed, store.dim().get(), pq)?;

        let mut lists = Vec::with_capacity(coarse.len());
        lists.resize_with(coarse.len(), || List {
            rows: Vec::new(),
            codes: Vec::new(),
        });

        for row in 0..store.len() {
            let Some(vector) = store.get(Ordinal::from_row(row as u32)) else {
                continue;
            };
            let list = coarse.assign(vector);
            let residual = coarse.residual(vector, list);
            lists[list].rows.push(row as u32);
            lists[list]
                .codes
                .extend_from_slice(&codebook.encode(&residual)?);
        }

        let assigned: usize = lists.iter().map(|l| l.rows.len()).sum();
        logger::info!("ivf-pq built").with_data(&serde_json::json!({
            fields::INDEX_KIND: "ivf-pq",
            fields::ROWS: assigned,
            fields::LEVELS: coarse.len(),
            fields::DURATION_SECONDS: started.elapsed().as_secs_f64(),
        }));

        Ok(Self {
            coarse,
            codebook,
            lists,
            params,
            rescore: 4,
            meter: Meter::disabled(),
        })
    }

    /// Search `nprobe` lists instead, without rebuilding.
    pub fn with_nprobe(mut self, nprobe: usize) -> Self {
        self.params = self.params.with_nprobe(nprobe);
        self
    }

    /// Rescore `k * factor` candidates exactly before returning `k`.
    ///
    /// PQ scores are approximate, so the ordering near the cut is the least
    /// trustworthy part of the result. Rescoring a few multiples of `k` against
    /// the true vectors recovers most of what quantization cost, for the price
    /// of that many full-width distance computations.
    ///
    /// A factor of one disables it, which is the right setting only when the
    /// stored vectors are unavailable.
    pub fn with_rescore(mut self, factor: usize) -> Self {
        self.rescore = factor.max(1);
        self
    }

    /// Bytes of code per row, which is what PQ actually buys.
    pub fn bytes_per_row(&self) -> usize {
        self.codebook.m()
    }
}

/// Residuals sampled for codebook training.
fn sample_residuals(store: &dyn VectorStore, coarse: &Coarse, params: &IvfParams) -> Vec<Vec<f32>> {
    let rows = store.len();
    let wanted = params.sample.min(rows).max(1);
    let stride = (rows / wanted).max(1);

    let mut out = Vec::new();
    let mut row = 0usize;
    while row < rows && out.len() < wanted {
        if let Some(vector) = store.get(Ordinal::from_row(row as u32)) {
            let list = coarse.assign(vector);
            out.push(coarse.residual(vector, list));
        }
        row += stride;
    }
    out
}

#[cfg(test)]
#[path = "pq_test.rs"]
mod tests;
