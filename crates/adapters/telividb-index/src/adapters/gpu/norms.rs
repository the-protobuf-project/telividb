//! Row norms, for the L2 expansion.
//!
//! Split from `gguf.rs` because that file is about the corpus's *format* — how
//! rows are written to and read from a GGUF tensor — while this is a derived
//! quantity computed from them at search time.

use super::gguf::Corpus;
use candle_core::Tensor;

impl Corpus {
    /// `‖row‖²` for every row, computed on first use.
    ///
    /// Once per corpus rather than once per query: the norms depend only on
    /// the stored vectors, so recomputing them per query would add a full pass
    /// over the corpus to every search — which is the entire cost the single
    /// matmul exists to avoid.
    pub fn row_norms(&self) -> telividb_core::Result<&Tensor> {
        if let Some(norms) = self.row_norms.get() {
            return Ok(norms);
        }

        let computed = self
            .tensor
            .sqr()
            .and_then(|squared| squared.sum_keepdim(candle_core::D::Minus1))
            .and_then(|sums| sums.t())
            .and_then(|row| row.contiguous())
            .map_err(|e| telividb_core::Error::GpuIndex {
                reason: e.to_string(),
            })?;

        // A concurrent caller may have won the race; either tensor is correct,
        // so whichever landed first is the one everyone uses.
        let _ = self.row_norms.set(computed);
        self.row_norms
            .get()
            .ok_or_else(|| telividb_core::Error::GpuIndex {
                reason: "row norms were not stored".to_owned(),
            })
    }
}
