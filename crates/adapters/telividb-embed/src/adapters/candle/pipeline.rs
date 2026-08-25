//! The embedding pipeline: tokenize, schedule, forward, normalize.
//!
//! Split from `inferencer.rs` because that file is the *registry* — which
//! models are resident and how one is resolved — while this is what happens to
//! a batch of text once a model has been chosen. The two change for different
//! reasons.

use super::batch;
use super::inferencer::CandleInferencer;
use super::schedule;
use crate::domain::{ModelId, Task};
use crate::error::Result;
use crate::ports::Inferencer;
use candle_core::Tensor;
use telividb_core::Dim;

/// Tokens per batch, counting padding.
///
/// `rows * padded_length`, which is what actually drives memory and time.
/// Chosen from measurement rather than theory: on an M-series GPU this model
/// sustains roughly 7,600 tokens per second, and batches around this size keep
/// it busy without making any single dispatch large enough to stall.
const TOKEN_BUDGET: usize = 16_384;

/// Rows per batch, whatever the budget allows.
///
/// At very short lengths the budget alone would permit thousands, and per-row
/// overhead starts to dominate before it is reached.
const MAX_ROWS: usize = 64;

impl Inferencer for CandleInferencer {
    fn embed(&self, model: &ModelId, task: Task, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let model = self.resolve(model)?;
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let encoder = model.encoder();
        // The model's context is a hard ceiling — its position embeddings do
        // not reach further — and the cap, when set, lowers it further.
        let context = match self.max_tokens {
            Some(cap) => cap.min(encoder.context()),
            None => encoder.context(),
        };
        let rows = batch::tokenize(model.tokenizer(), texts, task, context)?;

        // Batched by length rather than in input order. A batch is padded to
        // its longest member, so mixing a short sentence with a long abstract
        // computes the sentence at the abstract's length — see `schedule`.
        let lengths: Vec<usize> = rows.iter().map(Vec::len).collect();
        let plan = schedule::plan(&lengths, TOKEN_BUDGET, MAX_ROWS);

        // Placed back by index, so the caller still gets one vector per input
        // in input order — the reordering above is invisible to them.
        let mut out: Vec<Vec<f32>> = vec![Vec::new(); texts.len()];
        for group in plan {
            let slices: Vec<&[u32]> = group.iter().map(|i| rows[*i].as_slice()).collect();
            let batch = batch::to_tensors(&slices, encoder.device())?;
            let pooled = encoder.forward(&batch.ids, &batch.attention, model.pooling)?;

            for (index, vector) in group.into_iter().zip(normalize(&pooled)?.to_vec2()?) {
                out[index] = vector;
            }
        }
        Ok(out)
    }

    fn dim(&self, model: &ModelId) -> Result<Dim> {
        self.resolve(model)?.dim()
    }

    fn is_resident(&self, model: &ModelId) -> bool {
        self.resolve(model).is_ok()
    }
}

/// Scale each row to unit length.
///
/// Done here, once, rather than left to the caller. The storage layer's cosine
/// path is dot-product over pre-normalized vectors (see CLAUDE.md's cosine
/// note), so an un-normalized vector reaching it does not error — it ranks by
/// magnitude as much as by direction, which looks like a quality problem
/// rather than a bug.
///
/// The clamp guards a genuinely zero row, which a text that tokenizes to
/// nothing can produce: dividing by its zero norm yields `NaN`, and a `NaN`
/// vector silently poisons every comparison it takes part in.
fn normalize(xs: &Tensor) -> candle_core::Result<Tensor> {
    let norm = xs
        .sqr()?
        .sum_keepdim(candle_core::D::Minus1)?
        .sqrt()?
        .clamp(1e-12, f64::INFINITY)?;
    xs.broadcast_div(&norm)
}
