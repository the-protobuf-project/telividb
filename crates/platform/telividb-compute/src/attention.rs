//! The two operations attention needs that are not plain arithmetic.
//!
//! Both are *fused* in ggml rather than composed from smaller steps, and taking
//! the fused form is not an optimization detail — it is what keeps the scores
//! matrix from being written to memory three times on the hot path.

use crate::error::Result;
use crate::sys;
use crate::tensor::Tensor;

/// ggml's rotary mode for a plain, non-interleaved (GPT-NeoX / "normal")
/// rotation — the half-split convention.
///
/// Named rather than passed as a bare `0` because the alternative convention
/// pairs *adjacent* elements instead, and choosing wrong leaves every vector
/// well-formed while scrambling the positional signal completely. That failure
/// has already been paid for once in this codebase.
const ROPE_MODE_NORMAL: i32 = 0;

impl<'c, 'b> Tensor<'c, 'b> {
    /// Scaled, masked softmax over `ne[0]` — the attention weights.
    ///
    /// One call rather than `scale` then `add(mask)` then `softmax`: ggml fuses
    /// all three, so the scores matrix is read once instead of three times.
    ///
    /// `mask` is **additive and applied before the softmax**, which is the only
    /// correct order. A large negative sends a padded position to ~0 weight;
    /// zeroing the weights *after* the softmax instead leaves the row
    /// normalized over padding, so every real token is silently scaled down by
    /// whatever fraction of the row was padding — a bug that varies with batch
    /// composition and is therefore maddening to reproduce.
    ///
    /// `max_bias` is ALiBi's slope and is zero for an encoder.
    pub fn masked_softmax(&self, mask: Option<&Self>, scale: f32) -> Result<Self> {
        let mask = mask.map_or(std::ptr::null_mut(), |m| m.raw());
        // SAFETY: `raw` is a live node; `mask` is either null (which ggml
        // accepts, meaning no mask) or another live node in the same context.
        self.wrap("soft_max_ext", unsafe {
            sys::ggml_soft_max_ext(self.ctx().raw, self.raw(), mask, scale, 0.0)
        })
    }

    /// Rotary position embedding, applied to `n_dims` of each head.
    ///
    /// `positions` is an `i32` tensor, one entry per token. Applied to queries
    /// and keys **only, never values** — rotating V mixes content with position
    /// and produces plausible nonsense.
    ///
    /// `base` is the frequency base the model was trained with (10000 for most
    /// BERT-family rotary models); reading it from the GGUF rather than assuming
    /// is what keeps a model with an extended context working.
    pub fn rope(&self, positions: &Self, n_dims: usize, base: f32) -> Result<Self> {
        // SAFETY: both nodes are live in this context. The null `c` is ggml's
        // "no frequency scaling factors", and the trailing parameters are the
        // documented neutral values: no YaRN extension, unit attention scale.
        self.wrap("rope_ext", unsafe {
            sys::ggml_rope_ext(
                self.ctx().raw,
                self.raw(),
                positions.raw(),
                std::ptr::null_mut(),
                n_dims as i32,
                ROPE_MODE_NORMAL,
                0,
                base,
                1.0,
                0.0,
                1.0,
                32.0,
                1.0,
            )
        })
    }
}
