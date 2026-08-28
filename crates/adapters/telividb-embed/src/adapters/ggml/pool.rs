//! Collapsing a sequence of token vectors into one.
//!
//! Split from `encoder.rs` because it is the only step that happens *after* the
//! graph has run — everything there records operations, this reads results. It
//! is also where the most consequential arithmetic bug in an encoder lives, so
//! it is worth finding on its own.

use super::encoder::Encoder;
use crate::domain::Pooling;

impl Encoder {
    /// Collapse `hidden * tokens` to one vector per row.
    ///
    /// Mask-aware and divided by the **real** token count, not the padded
    /// width: dividing by the width shrinks every short text by however much
    /// padding it happened to receive, which varies with batch composition.
    pub(super) fn pool(
        &self,
        raw: &[f32],
        attention: &[u32],
        rows: usize,
        width: usize,
        pooling: Pooling,
    ) -> Vec<Vec<f32>> {
        let hidden = self.config.hidden;
        (0..rows)
            .map(|row| {
                let base = row * width;
                match pooling {
                    Pooling::Cls => raw[base * hidden..(base + 1) * hidden].to_vec(),
                    Pooling::Mean => {
                        let mut sum = vec![0f32; hidden];
                        let mut kept = 0f32;
                        for t in 0..width {
                            if attention[base + t] == 0 {
                                continue;
                            }
                            kept += 1.0;
                            let at = (base + t) * hidden;
                            for (s, v) in sum.iter_mut().zip(&raw[at..at + hidden]) {
                                *s += v;
                            }
                        }
                        let by = kept.max(1.0);
                        sum.iter().map(|v| v / by).collect()
                    }
                }
            })
            .collect()
    }
}
