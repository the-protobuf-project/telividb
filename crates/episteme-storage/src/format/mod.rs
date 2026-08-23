//! On-disk structures.

mod codec;
mod field_header;
mod header;

pub use codec::{Codec, DType};
pub use field_header::{FIELD_HEADER_BYTES, FIELD_VERSION, FieldHeader};
pub use header::{HEADER_BYTES, SEGMENT_MAGIC, SEGMENT_VERSION, SegmentHeader};
