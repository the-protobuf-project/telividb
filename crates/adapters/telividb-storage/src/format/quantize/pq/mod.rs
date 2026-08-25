//! Product quantization.

mod serialize;

pub use serialize::{decode_codebook, encode_codebook, encoded_len};
pub use telividb_distance::pq::{CENTROIDS, PqCodebook, PqParams};
