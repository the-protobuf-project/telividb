//! Scan tiers over quantized codes.
//!
//! Each of these implements [`ScanTier`](telividb_core::ScanTier) over one
//! codec, scoring against codes rather than decoding every row it will reject.

mod binary;
mod f16;
mod int8;
mod pq;

pub use binary::BinaryTier;
pub use f16::F16Tier;
pub use int8::Int8Tier;
pub use pq::PqTier;
