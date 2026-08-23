//! Compressing vectors for the wide scan.
//!
//! Quantization is a **storage codec, not an index concern**. The index sees a
//! vector store and never learns which codec is underneath — which is what lets
//! a custom search algorithm work over any of them.
//!
//! The point is the two-tier read: scan wide and cheap over `codes.bin`, then
//! rescore only the survivors at full precision from `raw.bin`. That buys more
//! recall per byte than any amount of graph tuning, because it lets far more
//! candidates be considered for the same memory budget.
//!
//! | Codec | Bytes per row at d=768 | Ratio |
//! |---|---|---|
//! | none (f32 only) | 3072 | 1× |
//! | int8 | 776 | ~4× |
//! | binary | 96 | 32× |
//!
//! Binary is coarse enough that it is only useful as a first pass in front of a
//! rerank; int8 is accurate enough to rank on directly for most models.

mod binary;
mod int8;

pub use binary::{BinaryCodes, hamming};
pub use int8::Int8Row;
